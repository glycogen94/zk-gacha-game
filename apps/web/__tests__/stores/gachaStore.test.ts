import * as wasmLoader from '@/lib/wasmLoader';
import { useGachaStore } from '@/stores/gachaStore';
// apps/web/__tests__/stores/gachaStore.test.ts
import { act } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

// WASM 모듈 모킹
vi.mock('@/lib/wasmLoader', () => ({
  loadWasmModule: vi.fn().mockResolvedValue({}),
  initGachaKeys: vi.fn().mockResolvedValue(undefined),
  generateGachaProof: vi.fn().mockResolvedValue(new Uint8Array([1, 2, 3, 4])),
  verifyGachaProof: vi.fn().mockResolvedValue(true),
}));

describe('useGachaStore', () => {
  // Store reset 함수 (테스트 간 상태 초기화를 위함)
  const resetStore = () => {
    const { resetPullState } = useGachaStore.getState();
    resetPullState();
    useGachaStore.setState({
      merkleRoot: null,
      itemMasterData: null,
      availableKeyUrls: [],
      isWasmInitialized: false,
      isKeysInitialized: false,
      isLoadingAssets: false,
      initializationError: null,
      inventory: [],
    });
  };

  beforeEach(() => {
    resetStore();
    vi.clearAllMocks();
  });

  afterEach(() => {
    resetStore();
  });

  // 전역 fetch 모킹
  const mockFetchResponses = {
    '/gacha/gacha_pk.bin': new Uint8Array([1, 2, 3]),
    '/gacha/gacha_vk.bin': new Uint8Array([4, 5, 6]),
    '/gacha/params.bin': new Uint8Array([7, 8, 9]),
    '/gacha/merkle_root.hex': 'deadbeef',
    '/gacha/items/key_list.txt':
      'gacha/items/item_0.json\ngacha/items/item_1.json',
    '/gacha/item_master.json': {
      '0x1234': {
        id: '0x1234',
        name: 'Pikachu',
        imageUrl: '/pokemon/pikachu.png',
        rarity: 'rare',
      },
    },
    'gacha/items/item_0.json': {
      itemIdHex: '0x1234',
      secretKeyHex: '0xabcd',
      merklePathNodesHex: ['0x1111', '0x2222'],
      leafSiblingHashHex: '0x3333',
      leafIndex: 0,
    },
  };

  // 전역 fetch 모킹 설정
  global.fetch = vi.fn().mockImplementation((url) => {
    const urlString = url.toString();

    // JSON 응답인 경우
    if (urlString.endsWith('.json')) {
      return Promise.resolve({
        ok: true,
        json: () => Promise.resolve(mockFetchResponses[urlString]),
        arrayBuffer: () => Promise.reject(new Error('Not an array buffer')),
        text: () => Promise.reject(new Error('Not a text')),
      });
    }

    // 텍스트 응답인 경우
    if (urlString.endsWith('.hex') || urlString.endsWith('.txt')) {
      return Promise.resolve({
        ok: true,
        text: () => Promise.resolve(mockFetchResponses[urlString]),
        json: () => Promise.reject(new Error('Not a JSON')),
        arrayBuffer: () => Promise.reject(new Error('Not an array buffer')),
      });
    }

    // 바이너리 응답인 경우
    if (urlString.endsWith('.bin')) {
      return Promise.resolve({
        ok: true,
        arrayBuffer: () => Promise.resolve(mockFetchResponses[urlString]),
        json: () => Promise.reject(new Error('Not a JSON')),
        text: () => Promise.reject(new Error('Not a text')),
      });
    }

    return Promise.reject(new Error(`Unhandled mock URL: ${urlString}`));
  });

  // 테스트 케이스
  it('initializes correctly with loadInitialAssets', async () => {
    const { loadInitialAssets } = useGachaStore.getState();

    // 초기 상태 확인
    expect(useGachaStore.getState().isKeysInitialized).toBe(false);

    // 함수 호출
    await act(async () => {
      await loadInitialAssets();
    });

    // 상태 변화 확인
    const state = useGachaStore.getState();
    expect(state.isWasmInitialized).toBe(true);
    expect(state.isKeysInitialized).toBe(true);
    expect(state.merkleRoot).toBe('deadbeef');
    expect(state.availableKeyUrls).toEqual([
      'gacha/items/item_0.json',
      'gacha/items/item_1.json',
    ]);
    expect(state.itemMasterData).toEqual({
      '0x1234': {
        id: '0x1234',
        name: 'Pikachu',
        imageUrl: '/pokemon/pikachu.png',
        rarity: 'rare',
      },
    });

    // WASM 로더 함수들이 호출되었는지 확인
    expect(wasmLoader.loadWasmModule).toHaveBeenCalledTimes(1);
    expect(wasmLoader.initGachaKeys).toHaveBeenCalledTimes(1);
  });

  it('performs a pull successfully', async () => {
    // 스토어 초기화
    useGachaStore.setState({
      isKeysInitialized: true,
      availableKeyUrls: ['gacha/items/item_0.json'],
      itemMasterData: {
        '0x1234': {
          id: '0x1234',
          name: 'Pikachu',
          imageUrl: '/pokemon/pikachu.png',
          rarity: 'rare',
        },
      },
    });

    const { performPull } = useGachaStore.getState();

    // 풀 수행
    await act(async () => {
      await performPull();
    });

    // 상태 확인
    const state = useGachaStore.getState();
    expect(state.isPulling).toBe(false);
    expect(state.pullResult).not.toBeNull();
    expect(state.pullResult?.itemDetails.name).toBe('Pikachu');

    // 인벤토리에 추가되었는지 확인
    expect(state.inventory.length).toBe(1);
    expect(state.inventory[0].name).toBe('Pikachu');
  });

  it('generates proof for last pull', async () => {
    // 스토어 초기화 (가챠 풀 완료 상태로)
    useGachaStore.setState({
      isKeysInitialized: true,
      merkleRoot: 'deadbeef',
      pullResult: {
        itemIdHex: '0x1234',
        secretKeyHex: '0xabcd',
        merklePathNodesHex: ['0x1111', '0x2222'],
        leafSiblingHashHex: '0x3333',
        leafIndex: 0,
        itemDetails: {
          id: '0x1234',
          name: 'Pikachu',
          imageUrl: '/pokemon/pikachu.png',
          rarity: 'rare',
        },
      },
    });

    const { generateProofForLastPull } = useGachaStore.getState();

    // 증명 생성
    await act(async () => {
      await generateProofForLastPull();
    });

    // 상태 확인
    const state = useGachaStore.getState();
    expect(state.isGeneratingProof).toBe(false);
    expect(state.proof).toBeInstanceOf(Uint8Array);
    expect(state.proofGenerationError).toBeNull();

    // WASM 함수가 호출되었는지 확인
    expect(wasmLoader.generateGachaProof).toHaveBeenCalledTimes(1);

    // 호출 인자 확인
    const expectedInputs = {
      merkleRoot: 'deadbeef',
      itemIdHex: '0x1234',
      secretKeyHex: '0xabcd',
      merklePathNodesHex: ['0x1111', '0x2222'],
      leafSiblingHashHex: '0x3333',
      leafIndex: 0,
    };
    expect(wasmLoader.generateGachaProof).toHaveBeenCalledWith(expectedInputs);
  });

  it('verifies proof correctly', async () => {
    // 스토어 초기화 (증명 생성 완료 상태로)
    const mockProof = new Uint8Array([1, 2, 3, 4]);
    useGachaStore.setState({
      isKeysInitialized: true,
      merkleRoot: 'deadbeef',
      proof: mockProof,
    });

    const { verifyProofForLastPull } = useGachaStore.getState();

    // 증명 검증
    await act(async () => {
      await verifyProofForLastPull();
    });

    // 상태 확인
    const state = useGachaStore.getState();
    expect(state.isVerifyingProof).toBe(false);
    expect(state.verificationResult).toBe(true);
    expect(state.verificationError).toBeNull();

    // WASM 함수가 호출되었는지 확인
    expect(wasmLoader.verifyGachaProof).toHaveBeenCalledTimes(1);
    expect(wasmLoader.verifyGachaProof).toHaveBeenCalledWith(
      'deadbeef',
      mockProof,
    );
  });

  it('handles loadInitialAssets errors correctly', async () => {
    // loadWasmModule에서 에러 발생하도록 설정
    vi.mocked(wasmLoader.loadWasmModule).mockRejectedValueOnce(
      new Error('WASM load failed'),
    );

    const { loadInitialAssets } = useGachaStore.getState();

    // 함수 호출
    await act(async () => {
      await loadInitialAssets();
    });

    // 상태 확인
    const state = useGachaStore.getState();
    expect(state.isWasmInitialized).toBe(false);
    expect(state.isKeysInitialized).toBe(false);
    expect(state.initializationError).toBe('WASM load failed');
  });

  it('handles proof generation errors correctly', async () => {
    // 스토어 초기화
    useGachaStore.setState({
      isKeysInitialized: true,
      merkleRoot: 'deadbeef',
      pullResult: {
        itemIdHex: '0x1234',
        secretKeyHex: '0xabcd',
        merklePathNodesHex: ['0x1111', '0x2222'],
        leafSiblingHashHex: '0x3333',
        leafIndex: 0,
        itemDetails: {
          id: '0x1234',
          name: 'Pikachu',
          imageUrl: '/pokemon/pikachu.png',
          rarity: 'rare',
        },
      },
    });

    // generateGachaProof에서 에러 발생하도록 설정
    vi.mocked(wasmLoader.generateGachaProof).mockRejectedValueOnce(
      new Error('Proof generation failed'),
    );

    const { generateProofForLastPull } = useGachaStore.getState();

    // 증명 생성 시도
    await act(async () => {
      await generateProofForLastPull();
    });

    // 상태 확인
    const state = useGachaStore.getState();
    expect(state.isGeneratingProof).toBe(false);
    expect(state.proof).toBeNull();
    expect(state.proofGenerationError).toBe('Proof generation failed');
  });
});
