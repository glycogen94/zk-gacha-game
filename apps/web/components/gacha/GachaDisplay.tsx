'use client';

import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button'; // Shadcn UI 버튼 컴포넌트 사용
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { useGachaStore } from '@/stores/gachaStore';
import { AlertCircle, CheckCircle2, Loader2, XCircle } from 'lucide-react';

// 트랜지션 효과를 위한 스타일 정의
const transitionStyle = 'transition-all duration-300 ease-in-out';

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
        <p>
          Initializing zero-knowledge proof system and cryptographic keys...
        </p>
      </div>
    );
  }

  return (
    <div className="flex flex-col items-center">
      {/* Security Disclaimer */}
      <Alert className="mb-6 max-w-md">
        <AlertCircle className="h-4 w-4" />
        <AlertTitle>Demo Environment</AlertTitle>
        <AlertDescription>
          This demo uses client-side proof generation for educational purposes.
          In a production environment, proofs would be generated server-side for
          proper security.
        </AlertDescription>
      </Alert>

      {/* Pull Button */}
      <Button
        onClick={handlePull}
        disabled={isPulling || isGeneratingProof || isVerifyingProof}
        className={`mb-6 px-6 py-3 text-lg bg-red-500 hover:bg-red-600 ${transitionStyle}`}
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

      {/* Pull Error Display */}
      {pullError && (
        <Alert variant="destructive" className="mb-6 max-w-md">
          <AlertCircle className="h-4 w-4" />
          <AlertTitle>Error</AlertTitle>
          <AlertDescription>{pullError}</AlertDescription>
        </Alert>
      )}

      {/* Result Display Area */}
      {pullResult && !isPulling && (
        <Card
          className={`w-full max-w-md ${transitionStyle} ${proof ? 'shadow-lg' : 'shadow'}`}
        >
          <CardHeader>
            <CardTitle className="text-center">You Pulled:</CardTitle>
          </CardHeader>
          <CardContent className="flex flex-col items-center">
            <div
              className={`w-40 h-56 mb-4 transform hover:scale-105 ${transitionStyle} overflow-hidden rounded-lg border-4`}
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
                className={`mb-2 w-full ${transitionStyle}`}
                variant="outline"
              >
                Generate Verification Proof
              </Button>
            )}
            {isGeneratingProof && (
              <div
                className={`flex items-center justify-center mb-2 w-full p-2 ${transitionStyle}`}
              >
                <Loader2 className="w-4 h-4 animate-spin mr-2" />
                <p>Generating Zero-Knowledge Proof...</p>
              </div>
            )}
            {proofGenerationError && (
              <Alert variant="destructive" className="mb-2">
                <AlertCircle className="h-4 w-4" />
                <AlertTitle>Proof Generation Failed</AlertTitle>
                <AlertDescription>{proofGenerationError}</AlertDescription>
              </Alert>
            )}

            {/* Proof Verification Button */}
            {proof && !isVerifyingProof && verificationResult === null && (
              <Button
                onClick={handleVerifyProof}
                disabled={isVerifyingProof}
                className={`mb-2 w-full ${transitionStyle}`}
                variant="outline"
              >
                Verify Proof
              </Button>
            )}
            {isVerifyingProof && (
              <div
                className={`flex items-center justify-center mb-2 w-full p-2 ${transitionStyle}`}
              >
                <Loader2 className="w-4 h-4 animate-spin mr-2" />
                <p>Verifying Zero-Knowledge Proof...</p>
              </div>
            )}
            {verificationError && (
              <Alert variant="destructive" className="mb-2">
                <AlertCircle className="h-4 w-4" />
                <AlertTitle>Verification Failed</AlertTitle>
                <AlertDescription>{verificationError}</AlertDescription>
              </Alert>
            )}

            {/* Verification Result */}
            {verificationResult !== null && (
              <div
                className={`flex items-center justify-center mb-2 w-full p-3 rounded-md ${transitionStyle} ${
                  verificationResult
                    ? 'bg-green-50 text-green-600 border border-green-200'
                    : 'bg-red-50 text-red-600 border border-red-200'
                }`}
              >
                {verificationResult ? (
                  <>
                    <CheckCircle2 className="w-5 h-5 mr-2" />
                    <span className="font-medium">
                      Verification Successful!
                    </span>
                  </>
                ) : (
                  <>
                    <XCircle className="w-5 h-5 mr-2" />
                    <span className="font-medium">Verification Failed!</span>
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
