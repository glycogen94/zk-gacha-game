'use client';

import GachaDisplay from '@/components/gacha/GachaDisplay'; // 이 컴포넌트도 나중에 생성할 예정입니다
import { useGachaStore } from '@/stores/gachaStore'; // 이 파일은 나중에 생성할 예정입니다
import { useEffect } from 'react';

export default function GachaPage() {
  // Select each piece of state individually
  const loadInitialAssets = useGachaStore((state) => state.loadInitialAssets);
  const isLoadingAssets = useGachaStore((state) => state.isLoadingAssets);
  const initializationError = useGachaStore(
    (state) => state.initializationError,
  );
  // Also select isKeysInitialized for the useEffect logic directly
  const isKeysInitialized = useGachaStore((state) => state.isKeysInitialized);

  // Load assets once on component mount
  useEffect(() => {
    // Now you can use the state variables directly from the hook
    if (!isKeysInitialized && !isLoadingAssets) {
      loadInitialAssets();
    }
    // Dependencies are now primitives or stable functions, which is ideal
  }, [isKeysInitialized, isLoadingAssets, loadInitialAssets]);

  if (isLoadingAssets) {
    return (
      <div className="container mx-auto p-4 text-center">
        Loading Gacha Assets... Please Wait...
      </div>
    );
  }

  if (initializationError) {
    return (
      <div className="container mx-auto p-4 text-center text-red-500">
        Initialization Failed: {initializationError}
      </div>
    );
  }

  // Render the main gacha component once assets are loaded
  return (
    <div className="container mx-auto p-4">
      <h1 className="text-3xl font-bold mb-6 text-center">
        ZK Verifiable Pokémon Card Pack
      </h1>
      <GachaDisplay />
      {/* Add Inventory Link */}
      <div className="mt-8 text-center">
        <a href="/gacha/inventory" className="text-blue-500 hover:underline">
          View My Collection
        </a>
      </div>
    </div>
  );
}
