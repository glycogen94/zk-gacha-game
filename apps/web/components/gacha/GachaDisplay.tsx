'use client';

import { Button } from '@/components/ui/button'; // Shadcn UI 버튼 컴포넌트 사용
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { useGachaStore } from '@/stores/gachaStore';
import { CheckCircle2, Loader2, XCircle } from 'lucide-react';

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

export default function GachaDisplay() {
  const {
    isKeysInitialized,
    performPull,
    isPulling,
    pullResult,
    pullError,
    generateProofForLastPull,
    isGeneratingProof,
    proof,
    proofGenerationError,
    verifyProofForLastPull,
    isVerifyingProof,
    verificationResult,
    verificationError,
    resetPullState,
  } = useGachaStore();

  const handlePull = () => {
    resetPullState(); // Reset previous states before new pull
    performPull();
  };

  const handleGenerateProof = () => {
    generateProofForLastPull();
  };

  const handleVerifyProof = () => {
    verifyProofForLastPull();
  };

  if (!isKeysInitialized) {
    return (
      <div className="flex items-center justify-center p-6">
        <Loader2 className="w-6 h-6 animate-spin mr-2" />
        <p>Initializing cryptographic components...</p>
      </div>
    );
  }

  return (
    <div className="flex flex-col items-center">
      {/* Pull Button */}
      <Button
        onClick={handlePull}
        disabled={isPulling || isGeneratingProof || isVerifyingProof}
        className="mb-6 px-6 py-3 text-lg bg-red-500 hover:bg-red-600"
        size="lg"
      >
        {isPulling ? (
          <>
            <Loader2 className="w-5 h-5 animate-spin mr-2" />
            Opening Poké Ball...
          </>
        ) : (
          'Open Poké Ball'
        )}
      </Button>

      {pullError && (
        <div className="text-red-500 mb-6 p-2 bg-red-50 rounded w-full max-w-md text-center">
          {pullError}
        </div>
      )}

      {/* Result Display Area */}
      {pullResult && !isPulling && (
        <Card className="w-full max-w-md">
          <CardHeader>
            <CardTitle className="text-center">You Pulled:</CardTitle>
          </CardHeader>
          <CardContent className="flex flex-col items-center">
            <div
              className="w-40 h-56 mb-4 transform hover:scale-105 transition-transform duration-300 overflow-hidden rounded-lg border-4"
              style={{
                borderColor: getPokemonColorByRarity(
                  pullResult.itemDetails.rarity || 'common',
                ),
              }}
            >
              <div className="bg-gradient-to-b from-blue-100 to-blue-50 w-full h-full p-2 flex flex-col items-center justify-center">
                <div className="w-28 h-28 bg-gray-100 rounded-full overflow-hidden mb-2 flex items-center justify-center">
                  <img
                    src={
                      pullResult.itemDetails.imageUrl ||
                      'https://placehold.co/128x128?text=Pokemon'
                    }
                    alt={pullResult.itemDetails.name}
                    className="w-full h-full object-contain"
                  />
                </div>
                <h3 className="text-lg font-bold text-center mb-1">
                  {pullResult.itemDetails.name}
                </h3>
                <p
                  className={`text-sm font-semibold mb-1 ${getRarityTextColor(pullResult.itemDetails.rarity || 'common')}`}
                >
                  {pullResult.itemDetails.rarity?.toUpperCase() || 'COMMON'}
                </p>
              </div>
            </div>
            <div className="w-full bg-gray-50 p-2 rounded text-xs overflow-hidden mb-4">
              <p className="truncate">Item ID: {pullResult.itemIdHex}</p>
            </div>

            {/* Proof Generation Button */}
            {!proof && !isGeneratingProof && (
              <Button
                onClick={handleGenerateProof}
                disabled={isGeneratingProof || isVerifyingProof}
                className="mb-2 w-full"
                variant="outline"
              >
                Generate Verification Proof
              </Button>
            )}
            {isGeneratingProof && (
              <div className="flex items-center mb-2">
                <Loader2 className="w-4 h-4 animate-spin mr-2" />
                <p>Generating Proof...</p>
              </div>
            )}
            {proofGenerationError && (
              <div className="text-red-500 mb-2 text-sm">
                Proof Generation Error: {proofGenerationError}
              </div>
            )}

            {/* Proof Verification Button */}
            {proof && !isVerifyingProof && verificationResult === null && (
              <Button
                onClick={handleVerifyProof}
                disabled={isVerifyingProof}
                className="mb-2 w-full"
                variant="outline"
              >
                Verify Proof
              </Button>
            )}
            {isVerifyingProof && (
              <div className="flex items-center mb-2">
                <Loader2 className="w-4 h-4 animate-spin mr-2" />
                <p>Verifying Proof...</p>
              </div>
            )}
            {verificationError && (
              <div className="text-red-500 mb-2 text-sm">
                Verification Error: {verificationError}
              </div>
            )}

            {/* Verification Result */}
            {verificationResult !== null && (
              <div
                className={`flex items-center mb-2 ${verificationResult ? 'text-green-600' : 'text-red-600'}`}
              >
                {verificationResult ? (
                  <>
                    <CheckCircle2 className="w-5 h-5 mr-2" />
                    Verification Successful!
                  </>
                ) : (
                  <>
                    <XCircle className="w-5 h-5 mr-2" />
                    Verification Failed!
                  </>
                )}
              </div>
            )}
          </CardContent>
          <CardFooter className="flex justify-center">
            <Button onClick={resetPullState} variant="ghost" size="sm">
              Reset
            </Button>
          </CardFooter>
        </Card>
      )}
    </div>
  );
}
