'use client';
import { useGachaStore } from '@/stores/gachaStore';
import Link from 'next/link';

// Helper functions for Pokémon card styling
const getPokemonColorByRarity = (rarity: string): string => {
  switch (rarity.toLowerCase()) {
    case 'legendary':
      return '#FFD700'; // Gold
    case 'rare':
      return '#0000FF'; // Blue
    case 'uncommon':
      return '#008000'; // Green
    default:
      return '#A9A9A9'; // Gray for common
  }
};

const getRarityTextColor = (rarity: string): string => {
  switch (rarity.toLowerCase()) {
    case 'legendary':
      return 'text-amber-500';
    case 'rare':
      return 'text-blue-600';
    case 'uncommon':
      return 'text-green-600';
    default:
      return 'text-gray-500';
  }
};

export default function InventoryPage() {
  const inventory = useGachaStore((state) => state.inventory);

  return (
    <div className="container mx-auto p-4">
      <h1 className="text-3xl font-bold mb-6 text-center">
        My Pokémon Card Collection
      </h1>

      <div className="mb-4">
        <Link href="/gacha" className="text-blue-500 hover:underline">
          &larr; Back to Gacha
        </Link>
      </div>

      {inventory.length === 0 ? (
        <div className="text-center py-10 bg-gray-50 rounded-lg">
          <p className="text-gray-500">
            Your collection is empty. Pull some Pokémon cards!
          </p>
        </div>
      ) : (
        <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4">
          {inventory.map((item, index) => (
            <div
              key={`${item.id}-${index}`}
              className="transform hover:scale-105 transition-transform duration-300"
            >
              <div
                className="w-full rounded-lg border-4 overflow-hidden"
                style={{
                  borderColor: getPokemonColorByRarity(item.rarity || 'common'),
                }}
              >
                <div className="bg-gradient-to-b from-blue-100 to-blue-50 p-2 flex flex-col items-center">
                  <div className="w-24 h-24 bg-gray-100 rounded-full overflow-hidden mb-2 flex items-center justify-center">
                    <img
                      src={
                        item.imageUrl ||
                        'https://placehold.co/100x100?text=Pokemon'
                      }
                      alt={item.name}
                      className="w-full h-full object-contain"
                    />
                  </div>
                  <h3 className="text-sm font-bold text-center mb-1">
                    {item.name}
                  </h3>
                  <p
                    className={`text-xs font-semibold ${getRarityTextColor(item.rarity || 'common')}`}
                  >
                    {item.rarity?.toUpperCase() || 'COMMON'}
                  </p>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
