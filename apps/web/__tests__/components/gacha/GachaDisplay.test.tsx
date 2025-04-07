import GachaDisplay from '@/components/gacha/GachaDisplay';
import { useGachaStore } from '@/stores/gachaStore';
// apps/web/__tests__/components/gacha/GachaDisplay.test.tsx
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

// Mock the gacha store
vi.mock('@/stores/gachaStore', () => ({
  useGachaStore: vi.fn(),
}));

// Define the mock implementation type
const mockStore = {
  isKeysInitialized: true,
  performPull: vi.fn(),
  isPulling: false,
  pullResult: null as {
    itemIdHex: string;
    secretKeyHex: string;
    merklePathNodesHex: string[];
    leafSiblingHashHex: string;
    leafIndex: number;
    itemDetails: {
      id: string;
      name: string;
      imageUrl: string;
      rarity?: string;
    };
  } | null,
  pullError: null as string | null,
  generateProofForLastPull: vi.fn(),
  isGeneratingProof: false,
  proof: null as Uint8Array | null,
  proofGenerationError: null as string | null,
  verifyProofForLastPull: vi.fn(),
  isVerifyingProof: false,
  verificationResult: null as boolean | null,
  verificationError: null as string | null,
  resetPullState: vi.fn(),
};

describe('GachaDisplay', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    // Default mock implementation
    vi.mocked(useGachaStore).mockImplementation(() => mockStore);
  });

  it('renders loading state when keys are not initialized', () => {
    vi.mocked(useGachaStore).mockImplementation(() => ({
      ...mockStore,
      isKeysInitialized: false,
    }));

    render(<GachaDisplay />);
    expect(
      screen.getByText(/initializing zero-knowledge proof system/i),
    ).toBeInTheDocument();
  });

  it('renders pull button in normal state', () => {
    render(<GachaDisplay />);

    // Should display the security disclaimer
    expect(screen.getByText(/demo environment/i)).toBeInTheDocument();

    // Should display pull button
    const pullButton = screen.getByRole('button', { name: /open poké ball/i });
    expect(pullButton).toBeInTheDocument();
    expect(pullButton).not.toBeDisabled();
  });

  it('displays loading state when pulling', () => {
    vi.mocked(useGachaStore).mockImplementation(() => ({
      ...mockStore,
      isPulling: true,
    }));

    render(<GachaDisplay />);
    expect(screen.getByText(/opening poké ball/i)).toBeInTheDocument();
  });

  it('displays pull error when there is one', () => {
    vi.mocked(useGachaStore).mockImplementation(() => ({
      ...mockStore,
      pullError: 'Failed to fetch item data',
    }));

    render(<GachaDisplay />);
    expect(screen.getByText(/failed to fetch item data/i)).toBeInTheDocument();
  });

  it('displays pulled item after successful pull', () => {
    vi.mocked(useGachaStore).mockImplementation(() => ({
      ...mockStore,
      pullResult: {
        itemIdHex: '0x1234',
        secretKeyHex: '0xabcd',
        merklePathNodesHex: [],
        leafSiblingHashHex: '0xef',
        leafIndex: 0,
        itemDetails: {
          id: '0x1234',
          name: 'Pikachu',
          imageUrl: '/pokemon/pikachu.png',
          rarity: 'rare',
        },
      },
    }));

    render(<GachaDisplay />);
    expect(screen.getByText(/you pulled/i)).toBeInTheDocument();
    expect(screen.getByText(/pikachu/i)).toBeInTheDocument();
    expect(screen.getByText(/rare/i)).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: /generate verification proof/i }),
    ).toBeInTheDocument();
  });

  it('shows proof generation state', () => {
    vi.mocked(useGachaStore).mockImplementation(() => ({
      ...mockStore,
      pullResult: {
        itemIdHex: '0x1234',
        secretKeyHex: '0xabcd',
        merklePathNodesHex: [],
        leafSiblingHashHex: '0xef',
        leafIndex: 0,
        itemDetails: {
          id: '0x1234',
          name: 'Pikachu',
          imageUrl: '/pokemon/pikachu.png',
          rarity: 'rare',
        },
      },
      isGeneratingProof: true,
    }));

    render(<GachaDisplay />);
    expect(
      screen.getByText(/generating zero-knowledge proof/i),
    ).toBeInTheDocument();
  });

  it('shows proof verification button after proof is generated', () => {
    vi.mocked(useGachaStore).mockImplementation(() => ({
      ...mockStore,
      pullResult: {
        itemIdHex: '0x1234',
        secretKeyHex: '0xabcd',
        merklePathNodesHex: [],
        leafSiblingHashHex: '0xef',
        leafIndex: 0,
        itemDetails: {
          id: '0x1234',
          name: 'Pikachu',
          imageUrl: '/pokemon/pikachu.png',
          rarity: 'rare',
        },
      },
      proof: new Uint8Array([1, 2, 3, 4]),
    }));

    render(<GachaDisplay />);
    expect(
      screen.getByRole('button', { name: /verify proof/i }),
    ).toBeInTheDocument();
  });

  it('shows verification result when proof is verified successfully', () => {
    vi.mocked(useGachaStore).mockImplementation(() => ({
      ...mockStore,
      pullResult: {
        itemIdHex: '0x1234',
        secretKeyHex: '0xabcd',
        merklePathNodesHex: [],
        leafSiblingHashHex: '0xef',
        leafIndex: 0,
        itemDetails: {
          id: '0x1234',
          name: 'Pikachu',
          imageUrl: '/pokemon/pikachu.png',
          rarity: 'rare',
        },
      },
      proof: new Uint8Array([1, 2, 3, 4]),
      verificationResult: true,
    }));

    render(<GachaDisplay />);
    expect(screen.getByText(/verification successful/i)).toBeInTheDocument();
  });

  it('shows failed verification result when proof verification fails', () => {
    vi.mocked(useGachaStore).mockImplementation(() => ({
      ...mockStore,
      pullResult: {
        itemIdHex: '0x1234',
        secretKeyHex: '0xabcd',
        merklePathNodesHex: [],
        leafSiblingHashHex: '0xef',
        leafIndex: 0,
        itemDetails: {
          id: '0x1234',
          name: 'Pikachu',
          imageUrl: '/pokemon/pikachu.png',
          rarity: 'rare',
        },
      },
      proof: new Uint8Array([1, 2, 3, 4]),
      verificationResult: false,
    }));

    render(<GachaDisplay />);
    expect(screen.getByText(/verification failed/i)).toBeInTheDocument();
  });

  it('calls performPull when button is clicked', async () => {
    render(<GachaDisplay />);

    const pullButton = screen.getByRole('button', { name: /open poké ball/i });
    fireEvent.click(pullButton);

    expect(mockStore.resetPullState).toHaveBeenCalledTimes(1);
    expect(mockStore.performPull).toHaveBeenCalledTimes(1);
  });

  it('calls generateProofForLastPull when generate proof button is clicked', async () => {
    vi.mocked(useGachaStore).mockImplementation(() => ({
      ...mockStore,
      pullResult: {
        itemIdHex: '0x1234',
        secretKeyHex: '0xabcd',
        merklePathNodesHex: [],
        leafSiblingHashHex: '0xef',
        leafIndex: 0,
        itemDetails: {
          id: '0x1234',
          name: 'Pikachu',
          imageUrl: '/pokemon/pikachu.png',
          rarity: 'rare',
        },
      },
    }));

    render(<GachaDisplay />);

    const generateButton = screen.getByRole('button', {
      name: /generate verification proof/i,
    });
    fireEvent.click(generateButton);

    expect(mockStore.generateProofForLastPull).toHaveBeenCalledTimes(1);
  });

  it('calls verifyProofForLastPull when verify button is clicked', async () => {
    vi.mocked(useGachaStore).mockImplementation(() => ({
      ...mockStore,
      pullResult: {
        itemIdHex: '0x1234',
        secretKeyHex: '0xabcd',
        merklePathNodesHex: [],
        leafSiblingHashHex: '0xef',
        leafIndex: 0,
        itemDetails: {
          id: '0x1234',
          name: 'Pikachu',
          imageUrl: '/pokemon/pikachu.png',
          rarity: 'rare',
        },
      },
      proof: new Uint8Array([1, 2, 3, 4]),
    }));

    render(<GachaDisplay />);

    const verifyButton = screen.getByRole('button', { name: /verify proof/i });
    fireEvent.click(verifyButton);

    expect(mockStore.verifyProofForLastPull).toHaveBeenCalledTimes(1);
  });
});
