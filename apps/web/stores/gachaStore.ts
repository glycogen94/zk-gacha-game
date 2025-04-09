import { create } from 'zustand';
import { createJSONStorage, persist } from 'zustand/middleware';
import {
  generateGachaProof,
  initGachaKeys,
  loadWasmModule,
  verifyGachaProof,
} from '@/lib/wasmLoader';

export interface ItemDetails {
  id: string;
  name: string;
  imageUrl: string;
  rarity?: string;
}

interface PullResultData {
  itemIdHex: string;
  secretKeyHex: string;
  merklePathNodesHex: string[];
  leafSiblingHashHex: string;
  leafIndex: number;
  itemDetails: ItemDetails;
}

export interface GachaState {
  merkleRoot: string | null;
  itemMasterData: Record<string, ItemDetails> | null;
  availableKeyUrls: string[];
  isWasmInitialized: boolean;
  isKeysInitialized: boolean;
  isLoadingAssets: boolean;
  initializationError: string | null;
  isPulling: boolean;
  pullResult: PullResultData | null;
  pullError: string | null;
  isGeneratingProof: boolean;
  proof: Uint8Array | null;
  proofGenerationError: string | null;
  isVerifyingProof: boolean;
  verificationResult: boolean | null;
  verificationError: string | null;
  inventory: ItemDetails[];
  loadInitialAssets: () => Promise<void>;
  performPull: () => Promise<void>;
  generateProofForLastPull: () => Promise<void>;
  verifyProofForLastPull: () => Promise<void>;
  resetPullState: () => void;
}

export const useGachaStore = create<GachaState>()(
  persist(
    (set, get) => ({
      merkleRoot: null,
      itemMasterData: null,
      availableKeyUrls: [],
      isWasmInitialized: false,
      isKeysInitialized: false,
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

      loadInitialAssets: async () => {
        const state = get();
        if (state.isLoadingAssets || state.isKeysInitialized) {
          console.log('Assets already loading or initialized.');
          return;
        }

        set({ isLoadingAssets: true, initializationError: null });
        console.log('Starting initial asset load...');

        try {
          await loadWasmModule();
          set({ isWasmInitialized: true });
          console.log('WASM loader finished.');

          console.log('Fetching assets...');
          const [pkRes, vkRes, paramsRes, rootRes, keyListRes, masterDataRes] =
            await Promise.all([
              fetch('/gacha/gacha_pk.bin'),
              fetch('/gacha/gacha_vk.bin'),
              fetch('/gacha/params.bin'),
              fetch('/gacha/merkle_root.hex'),
              fetch('/gacha/items/key_list.txt'),
              fetch('/gacha/item_master.json'),
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
              const trimmed = line.trim();
              return trimmed.replace(/^apps\/web\/public\//, '');
            })
            .filter((line) => line !== '');

          if (availableKeyUrls.length === 0) {
            console.warn('Warning: key_list.txt seems empty or invalid.');
          }

          console.log('Initializing WASM with keys and params via loader...');
          await initGachaKeys(pkBytes, vkBytes, paramsBytes);
          set({ isKeysInitialized: true });
          console.log('WASM keys initialized.');

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
            isWasmInitialized: false,
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
        if (isPulling || isLoadingAssets || !isKeysInitialized) return;
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
          const fullUrl = `/${selectedUrlPath}`;

          console.log(`Fetching item data from: ${fullUrl}`);
          const response = await fetch(fullUrl);
          if (!response.ok) {
            throw new Error(`Failed to fetch data from ${fullUrl}`);
          }

          const itemProofJson: Omit<PullResultData, 'itemDetails'> =
            await response.json();
          if (!itemProofJson || typeof itemProofJson.itemIdHex === 'undefined') {
            set({
              pullError: 'Invalid pull data received. Check the JSON structure.',
            });
            return;
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
          const wasmInputs = {
            merkleRoot: merkleRoot,
            itemIdHex: pullResult.itemIdHex,
            secretKeyHex: pullResult.secretKeyHex,
            merklePathNodesHex: pullResult.merklePathNodesHex,
            leafSiblingHashHex: pullResult.leafSiblingHashHex,
            leafIndex: pullResult.leafIndex,
          };
          console.log('Inputs for proof generation (JS Object):', wasmInputs);

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
