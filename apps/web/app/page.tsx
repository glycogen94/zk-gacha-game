import Link from 'next/link';

export default function Home() {
  return (
    <main className="flex min-h-screen flex-col items-center justify-center p-24">
      <h1 className="text-4xl font-bold mb-8">ZK Pokémon Card Game Demo</h1>

      <p className="text-center max-w-xl mb-8">
        A demonstration of a verifiable Pokémon card pack opening using
        Zero-Knowledge Proofs (ZK-SNARKs). Client-side proof generation and
        verification happen directly in your browser using WebAssembly.
      </p>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <Link
          href="/gacha"
          className="bg-blue-500 hover:bg-blue-600 text-white font-semibold py-3 px-6 rounded-lg text-center transition-colors"
        >
          Try ZK Pokémon Demo
        </Link>

        <Link
          href="/gacha/inventory"
          className="bg-gray-800 hover:bg-gray-900 text-white font-semibold py-3 px-6 rounded-lg text-center transition-colors"
        >
          View Inventory
        </Link>
      </div>
    </main>
  );
}
