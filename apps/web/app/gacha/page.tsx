'use client';

import GachaDisplay from '@/components/gacha/GachaDisplay'; // 이 컴포넌트도 나중에 생성할 예정입니다
import { useGachaStore } from '@/stores/gachaStore'; // 이 파일은 나중에 생성할 예정입니다
import { useEffect } from 'react';

export default function GachaPage() {
  const { loadInitialAssets, isLoadingAssets, initializationError } =
    useGachaStore((state) => ({
      loadInitialAssets: state.loadInitialAssets,
      isLoadingAssets: state.isLoadingAssets,
      initializationError: state.initializationError,
    }));

  // Load assets once on component mount
  useEffect(() => {
    // isInitialized 값을 확인하여 이미 초기화된 경우 다시 호출하지 않음
    const { isKeysInitialized, isLoadingAssets } = useGachaStore.getState();
    if (!isKeysInitialized && !isLoadingAssets) {
      loadInitialAssets();
    }
  }, []);

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
