// apps/web/lib/wasmLoader.ts
import type * as Wasm from 'zk-circuits'; // WASM 패키지 타입 임포트

// 로드된 WASM 모듈 타입을 위한 별칭
type WasmModule = typeof Wasm;

let wasmModule: WasmModule | null = null;
let isLoading = false;
let loadingPromise: Promise<WasmModule> | null = null;

/**
 * WASM 모듈이 로드되지 않았으면 로드합니다. 동시 요청을 처리합니다.
 * 사용 가능한 경우 패닉 후크 설정을 호출합니다.
 * @returns 로드된 WASM 모듈로 확인되는 프로미스를 반환합니다.
 */
export const loadWasmModule = (): Promise<WasmModule> => {
  // 이미 로드되었으면 즉시 반환
  if (wasmModule) {
    return Promise.resolve(wasmModule);
  }
  // 로딩 중이면 기존 프로미스 반환
  if (isLoading && loadingPromise) {
    return loadingPromise;
  }

  isLoading = true;
  // 로딩 프로미스 생성
  loadingPromise = new Promise((resolve, reject) => {
    // 비동기 함수를 별도로 정의하고 즉시 호출
    const loadWasmAsync = async () => {
      try {
        console.log('Loading WASM module (zk-circuits)...');
        // WASM 모듈 동적 임포트
        const loaded: WasmModule = await import('zk-circuits');
        console.log('WASM module imported.');

        // 'default' 익스포트 함수 실행 (필요한 경우)
        if (typeof loaded.default === 'function') {
          console.log('Calling default export for initialization (if any)...');
          await loaded.default();
        }

        // 패닉 후크 설정 함수 호출 (존재하는 경우)
        if (typeof loaded.main_js === 'function') {
          try {
            loaded.main_js(); // #[wasm_bindgen(start)] 함수 호출
            console.log('Panic hook setup called.');
          } catch (e) {
            console.warn('Could not call main_js (panic hook setup):', e);
          }
        } else {
          console.warn(
            "WASM module does not export 'main_js' (expected for panic hook).",
          );
        }

        wasmModule = loaded; // 로드된 모듈 저장
        isLoading = false;
        console.log('WASM module loaded successfully.');
        resolve(loaded); // 프로미스 확인
      } catch (error) {
        console.error('Failed to load WASM module:', error);
        isLoading = false;
        loadingPromise = null; // 에러 시 프로미스 초기화
        reject(error); // 프로미스 거부
      }
    };

    // 정의한 비동기 함수 호출
    loadWasmAsync();
  });

  return loadingPromise;
};

/**
 * 이미 로드된 WASM 모듈을 반환합니다. 로드되지 않았으면 에러를 발생시킵니다.
 * 모듈이 이전에 로드되었음을 확신할 때 유용합니다.
 */
export const getLoadedWasmModule = (): WasmModule => {
  if (!wasmModule) {
    throw new Error('WASM module is not loaded. Call loadWasmModule() first.');
  }
  return wasmModule;
};

// --- 특정 WASM 함수 내보내기 (편의를 위해) ---
// 이 함수들은 모듈이 로드되었는지 확인한 후 내보낸 함수에 접근합니다.

/**
 * WASM 모듈에 암호화 키와 파라미터를 초기화합니다.
 * @param pkBytes 직렬화된 증명 키 바이트
 * @param vkBytes 직렬화된 검증 키 바이트
 * @param paramsBytes 직렬화된 Poseidon 파라미터 바이트
 */
export const initGachaKeys = async (
  pkBytes: Uint8Array,
  vkBytes: Uint8Array,
  paramsBytes: Uint8Array,
): Promise<void> => {
  const wasm = await loadWasmModule(); // 모듈 로드 보장
  // 함수 존재 여부 확인
  if (typeof wasm.init_gacha_keys !== 'function') {
    throw new Error("WASM module does not export 'init_gacha_keys'");
  }
  // WASM 함수 호출
  return wasm.init_gacha_keys(pkBytes, vkBytes, paramsBytes);
};

/**
 * 제공된 입력에 대한 ZK-SNARK 증명을 생성합니다.
 * @param inputs WasmGachaCircuitInputs와 일치하는 JavaScript 객체
 * @returns 직렬화된 증명 바이트 배열
 */
// Define input type for generateGachaProof
interface GachaCircuitInputs {
  merkleRoot: string;
  itemIdHex: string;
  secretKeyHex: string;
  merklePathNodesHex: string[];
  leafSiblingHashHex: string;
  leafIndex: number;
}

export const generateGachaProof = async (
  inputs: GachaCircuitInputs,
): Promise<Uint8Array> => {
  const wasm = await loadWasmModule();
  if (typeof wasm.generate_gacha_proof !== 'function') {
    throw new Error("WASM module does not export 'generate_gacha_proof'");
  }
  // WASM 함수에 JavaScript 객체 직접 전달
  // serde_wasm_bindgen이 JsValue -> WasmGachaCircuitInputs 변환을 처리함
  return wasm.generate_gacha_proof(inputs);
};

/**
 * 제공된 공개 입력 및 증명을 검증합니다.
 * @param merkleRootHex 공개 Merkle 루트 (헥스 문자열)
 * @param proofBytes 직렬화된 증명 바이트 배열
 * @returns 증명이 유효하면 true, 그렇지 않으면 false
 */
export const verifyGachaProof = async (
  merkleRootHex: string,
  proofBytes: Uint8Array,
): Promise<boolean> => {
  const wasm = await loadWasmModule();
  if (typeof wasm.verify_gacha_proof !== 'function') {
    throw new Error("WASM module does not export 'verify_gacha_proof'");
  }
  // WASM 함수 호출
  return wasm.verify_gacha_proof(merkleRootHex, proofBytes);
};
