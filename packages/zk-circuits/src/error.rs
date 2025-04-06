// packages/zk-circuits/src/error.rs
use ark_serialize::SerializationError;
use thiserror::Error;
use wasm_bindgen::JsValue;

#[derive(Error, Debug)]
pub enum GachaCircuitError {
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Arkworks Serialization error: {0}")]
    ArkSerialization(#[from] SerializationError),
    #[error("Deserialization error: {0}")]
    Deserialization(String),
    #[error("Arkworks synthesis error: {0}")]
    Synthesis(#[from] ark_relations::r1cs::SynthesisError),
    #[error("Proof generation failed: {0}")]
    ProofGeneration(String),
    #[error("Proof verification failed: {0}")]
    ProofVerification(String),
    #[error("Keys or Parameters not initialized")]
    NotInitialized,
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Data conversion error: {0}")]
    ConversionError(String),
    #[error("Hex decoding error: {0}")]
    HexError(#[from] hex::FromHexError),
    #[error("Setup error: {0}")]
    SetupError(String),
    #[error("Underlying crypto primitive error: {0}")]
    PrimitiveError(String), // Wrap errors from ark-crypto-primitives
}

// Convert GachaCircuitError to JsValue for WASM boundary
impl From<GachaCircuitError> for JsValue {
    fn from(error: GachaCircuitError) -> Self {
        JsValue::from_str(&error.to_string())
    }
}

// Allow '?' operator to convert ark_crypto_primitives::Error
impl From<ark_crypto_primitives::Error> for GachaCircuitError {
    fn from(err: ark_crypto_primitives::Error) -> Self {
        GachaCircuitError::PrimitiveError(err.to_string())
    }
}
