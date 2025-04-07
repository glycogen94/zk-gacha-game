// apps/web/stores/gachaStore.ts
import { create } from 'zustand';
import { createJSONStorage, persist } from 'zustand/middleware';
// Import types from the WASM package if needed elsewhere, but not directly for WASM calls now
// import type { WasmGachaCircuitInputs } from 'zk-circuits';

// Import functions from the WASM loader
import {
  generateGachaProof,
  getLoadedWasmModule, // Optional: Use if sure it's loaded
  initGachaKeys,
  loadWasmModule, // Call this once to ensure loading
  verifyGachaProof,
} from '@/lib/wasmLoader'; // Adjust path if needed

// Define types for clarity within the store
export interface ItemDetails {
  id: string;
  name: string;
  imageUrl: string;
  rarity?: string;
}

// Data structure expected from the fetched item_{index}.json file
interface PullResultData {
  itemIdHex: string;
  secretKeyHex: string;
  merklePathNodesHex: string[];
  leafSiblingHashHex: string;
  leafIndex: number;
  itemDetails: ItemDetails;
}

// The complete state managed by Zustand
export interface GachaState {
  // --- Static Assets & Initialization Status ---
  merkleRoot: string | null;
  itemMasterData: Record<string, ItemDetails> | null;
  availableKeyUrls: string[];
  isWasmInitialized: boolean; // Now tracks if loadWasmModule has resolved
  isKeysInitialized: boolean; // Tracks if initGachaKeys has been successfully called via loader
  isLoadingAssets: boolean;
  initializationError: string | null;

  // --- Gacha Process State ---
  isPulling: boolean;
  pullResult: PullResultData | null;
  pullError: string | null;

  // --- Proof State ---
  isGeneratingProof: boolean;
  proof: Uint8Array | null;
  proofGenerationError: string | null;

  // --- Verification State ---
  isVerifyingProof: boolean;
  verificationResult: boolean | null;
  verificationError: string | null;

  // --- User Data ---
  inventory: ItemDetails[];

  // --- Actions ---
  loadInitialAssets: () => Promise<void>;
  performPull: () => Promise<void>;
  generateProofForLastPull: () => Promise<void>;
  verifyProofForLastPull: () => Promise<void>;
  resetPullState: () => void;
}

export const useGachaStore = create<GachaState>()(
  persist(
    (set, get) => ({
      // --- Initial State Values ---
      merkleRoot: null,
      itemMasterData: null,
      availableKeyUrls: [],
      isWasmInitialized: false, // Initialized via loadInitialAssets
      isKeysInitialized: false, // Initialized via loadInitialAssets
      isLoadingAssets: false,
      initializationError: null,
      isPulling: false,
      pullResult: null,
      pullError: null,
      isGeneratingProof: false,
      proof: null,
      proofGenerationError: null,
      isVerifyingProof: false,
      verificationResult: null,
      verificationError: null,
      inventory: [],

      // --- Action Implementations ---
      loadInitialAssets: async () => {
        // 이미 로딩 중이거나 초기화되어 있는 경우 바로 종료
        const state = get();
        if (state.isLoadingAssets || state.isKeysInitialized) {
          console.log('Assets already loading or initialized.');
          return;
        }

        // 로딩 상태에 따라서만 상태 변경
        set({ isLoadingAssets: true, initializationError: null });
        console.log('Starting initial asset load...');

        try {
          // 1. Ensure WASM module is loaded (and panic hook is set) via the loader
          // This will only load it once. Subsequent calls resolve immediately.
          await loadWasmModule();
          set({ isWasmInitialized: true }); // Mark WASM module as ready
          console.log('WASM loader finished.');

          // 2. Fetch cryptographic assets and game data
          console.log('Fetching assets...');
          const [pkRes, vkRes, paramsRes, rootRes, keyListRes, masterDataRes] =
            await Promise.all([
              fetch('/gacha/gacha_pk.bin'),
              fetch('/gacha/gacha_vk.bin'),
              fetch('/gacha/params.bin'),
              fetch('/gacha/merkle_root.hex'), // Fetch the hex string
              fetch('/gacha/items/key_list.txt'), // Fetch the list of JSON file URLs
              fetch('/gacha/item_master.json'), // Fetch item details map
            ]);

          const responses = {
            pkRes,
            vkRes,
            paramsRes,
            rootRes,
            keyListRes,
            masterDataRes,
          };
          const failedFetches = Object.entries(responses)
            .filter(([, res]) => !res.ok)
            .map(([key]) => key.replace('Res', ''));

          if (failedFetches.length > 0) {
            throw new Error(
              `Failed to fetch initial assets: ${failedFetches.join(', ')}`,
            );
          }
          console.log('All assets fetched successfully.');

          // 3. Process fetched data
          const pkBytes = new Uint8Array(await pkRes.arrayBuffer());
          const vkBytes = new Uint8Array(await vkRes.arrayBuffer());
          const paramsBytes = new Uint8Array(await paramsRes.arrayBuffer());
          const merkleRootHex = (await rootRes.text()).trim();
          const keyListText = await keyListRes.text();
          const itemMaster = (await masterDataRes.json()) as Record<
            string,
            ItemDetails
          >;

          const availableKeyUrls = keyListText
            .split('\n')
            .map((line) => {
              // Remove the 'apps/web/public/' prefix if it exists
              const trimmed = line.trim();
              return trimmed.replace(/^apps\/web\/public\//, '');
            })
            .filter((line) => line !== '');

          if (availableKeyUrls.length === 0) {
            console.warn('Warning: key_list.txt seems empty or invalid.');
          }

          // 4. Initialize WASM keys/params using the loader's exported function
          console.log('Initializing WASM with keys and params via loader...');
          await initGachaKeys(pkBytes, vkBytes, paramsBytes);
          set({ isKeysInitialized: true }); // Mark keys as initialized in WASM
          console.log('WASM keys initialized.');

          // 5. Update state
          set({
            merkleRoot: merkleRootHex,
            availableKeyUrls: availableKeyUrls,
            itemMasterData: itemMaster,
            isLoadingAssets: false,
          });
          console.log('Gacha store initialized successfully.');
        } catch (error) {
          console.error('Initialization failed:', error);
          set({
            initializationError:
              error instanceof Error ? error.message : String(error),
            isLoadingAssets: false,
            isWasmInitialized: false, // Reset flags on error
            isKeysInitialized: false,
          });
        }
      },

      performPull: async () => {
        const {
          availableKeyUrls,
          itemMasterData,
          isPulling,
          isLoadingAssets,
          isKeysInitialized,
        } = get();
        if (isPulling || isLoadingAssets || !isKeysInitialized) return; // Prevent concurrent/premature pulls
        if (availableKeyUrls.length === 0 || !itemMasterData) {
          set({ pullError: 'Assets not loaded or no keys available.' });
          return;
        }
        set({
          isPulling: true,
          pullError: null,
          pullResult: null,
          proof: null,
          verificationResult: null,
          proofGenerationError: null,
          verificationError: null,
        });
        console.log('Performing gacha pull...');

        try {
          const randomIndex = Math.floor(
            Math.random() * availableKeyUrls.length,
          );
          const selectedUrlPath = availableKeyUrls[randomIndex];
          const fullUrl = `/${selectedUrlPath}`; // Assumes relative path from /public

          console.log(`Fetching item data from: ${fullUrl}`);
          const response = await fetch(fullUrl);
          if (!response.ok)
            throw new Error(`Fetch failed: ${response.status} for ${fullUrl}`);

          const itemProofJson: Omit<PullResultData, 'itemDetails'> =
            await response.json();

          // Basic Validation
          const requiredKeys: (keyof Omit<PullResultData, 'itemDetails'>)[] = [
            'itemIdHex',
            'secretKeyHex',
            'merklePathNodesHex',
            'leafSiblingHashHex',
            'leafIndex',
          ];
          if (
            requiredKeys.some((key) => !(key in itemProofJson)) ||
            !Array.isArray(itemProofJson.merklePathNodesHex) ||
            typeof itemProofJson.leafIndex !== 'number'
          ) {
            throw new Error(`Invalid item data structure from ${fullUrl}`);
          }

          const itemDetails = itemMasterData[itemProofJson.itemIdHex];
          let finalPullResult: PullResultData;

          if (!itemDetails) {
            console.warn(
              `Item details not found for ID: ${itemProofJson.itemIdHex}. Using placeholder.`,
            );
            const fallbackItem: ItemDetails = {
              id: `unknown-${itemProofJson.itemIdHex.substring(2, 8)}`,
              name: `Mystery Item #${itemProofJson.itemIdHex.substring(2, 8)}`,
              imageUrl: '/placeholder.png',
              rarity: 'unknown',
            };
            finalPullResult = { ...itemProofJson, itemDetails: fallbackItem };
            set((state) => ({ inventory: [...state.inventory, fallbackItem] }));
          } else {
            finalPullResult = { ...itemProofJson, itemDetails: itemDetails };
            set((state) => ({ inventory: [...state.inventory, itemDetails] }));
          }

          set({ pullResult: finalPullResult, isPulling: false });
          console.log('Pull successful:', finalPullResult.itemDetails.name);
        } catch (error) {
          console.error('Pull failed:', error);
          set({
            pullError: error instanceof Error ? error.message : String(error),
            isPulling: false,
          });
        }
      },

      generateProofForLastPull: async () => {
        const { pullResult, merkleRoot, isGeneratingProof, isKeysInitialized } =
          get();
        if (isGeneratingProof || !isKeysInitialized) return;
        if (!pullResult || !merkleRoot) {
          set({
            proofGenerationError:
              'Cannot generate proof: Missing pull data or Merkle root.',
          });
          return;
        }
        set({
          isGeneratingProof: true,
          proofGenerationError: null,
          proof: null,
          verificationResult: null,
          verificationError: null,
        });
        console.log('Generating ZK proof...');

        try {
          // Prepare the input object structure
          // This needs to exactly match the structure expected by the Rust
          // function that uses #[wasm_bindgen] and serde_wasm_bindgen::from_value
          const wasmInputs = {
            merkleRoot: merkleRoot,
            itemIdHex: pullResult.itemIdHex,
            secretKeyHex: pullResult.secretKeyHex,
            merklePathNodesHex: pullResult.merklePathNodesHex,
            leafSiblingHashHex: pullResult.leafSiblingHashHex,
            leafIndex: pullResult.leafIndex,
          };
          console.log('Inputs for proof generation (JS Object):', wasmInputs);

          // 변경된 부분: 객체를 직접 전달
          // JSON 문자열 변환 및 Uint8Array 변환 단계를 제거
          const proofBytes: Uint8Array = await generateGachaProof(wasmInputs);

          set({ proof: proofBytes, isGeneratingProof: false });
          console.log(
            'Proof generated successfully (bytes):',
            proofBytes.length,
          );
        } catch (error) {
          console.error('Proof generation failed:', error);
          set({
            proofGenerationError:
              error instanceof Error ? error.message : String(error),
            isGeneratingProof: false,
          });
        }
      },

      verifyProofForLastPull: async () => {
        const { proof, merkleRoot, isVerifyingProof, isKeysInitialized } =
          get();
        if (isVerifyingProof || !isKeysInitialized) return;
        if (!proof || !merkleRoot) {
          set({
            verificationError:
              'Cannot verify: No proof or Merkle root available.',
          });
          return;
        }
        set({
          isVerifyingProof: true,
          verificationError: null,
          verificationResult: null,
        });
        console.log('Verifying ZK proof...');

        try {
          // Call the WASM function via the loader's export
          const isValid: boolean = await verifyGachaProof(merkleRoot, proof);
          console.log('Verification result:', isValid);

          set({ verificationResult: isValid, isVerifyingProof: false });
        } catch (error) {
          console.error('Proof verification failed:', error);
          set({
            verificationError:
              error instanceof Error ? error.message : String(error),
            isVerifyingProof: false,
          });
        }
      },

      resetPullState: () => {
        console.log('Resetting pull state...');
        set({
          isPulling: false,
          pullResult: null,
          pullError: null,
          isGeneratingProof: false,
          proof: null,
          proofGenerationError: null,
          isVerifyingProof: false,
          verificationResult: null,
          verificationError: null,
        });
      },
    }),
    {
      name: 'gacha-game-storage',
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({ inventory: state.inventory }),
    },
  ),
);
