// packages/zk-circuits/src/error.rs
use ark_serialize::SerializationError;
use ark_relations::r1cs::SynthesisError; // Ensure this is imported if using SynthesisError
use thiserror::Error;
use wasm_bindgen::JsValue;
use ark_std::format; // Use ark_std's format macro
use hex::FromHexError;

#[derive(Error, Debug)]
pub enum GachaCircuitError {
    // Keep using #[error] for message structure, but be careful with direct formatting of external types
    #[error("Serialization failed: {0}")]
    Serialization(String), // Use specific String message from source

    // Use a specific variant for ArkSerialization, avoid direct #[from] if Display is problematic
    #[error("Arkworks serialization failed")]
    ArkSerialization(SerializationError), // Store the original error

    #[error("Deserialization failed: {0}")]
    Deserialization(String),

    // Use a specific variant for Arkworks SynthesisError
    #[error("Arkworks constraint synthesis failed")]
    ArkSynthesis(SynthesisError), // Store the original error

    #[error("Proof generation failed: {0}")]
    ProofGeneration(String),

    #[error("Proof verification failed: {0}")]
    ProofVerification(String),

    #[error("System not initialized (Keys or Parameters missing)")]
    NotInitialized,

    #[error("Invalid input provided: {0}")]
    InvalidInput(String),

    #[error("Data conversion error: {0}")]
    ConversionError(String),

    // Use a specific variant for HexError
    #[error("Hex decoding failed")]
    HexDecoding(FromHexError), // #[from] might be okay here, but conversion below is safer

    #[error("Setup error: {0}")]
    SetupError(String),

    // Use a specific variant for PrimitiveError
    #[error("Cryptographic primitive error")]
    Primitive(ark_crypto_primitives::Error), // Store the original error
}

// --- Custom From implementations to capture source ---

impl From<SerializationError> for GachaCircuitError {
    fn from(err: SerializationError) -> Self {
        GachaCircuitError::ArkSerialization(err)
    }
}

impl From<SynthesisError> for GachaCircuitError {
    fn from(err: SynthesisError) -> Self {
        GachaCircuitError::ArkSynthesis(err)
    }
}

impl From<FromHexError> for GachaCircuitError {
    fn from(err: FromHexError) -> Self {
        GachaCircuitError::HexDecoding(err)
    }
}

// Keep the #[from] for hex::FromHexError if it's known to be no_std compatible,
// otherwise, implement From manually like the others. Let's assume #[from] is okay for now.
// impl From<hex::FromHexError> for GachaCircuitError {
//     fn from(err: hex::FromHexError) -> Self {
//         GachaCircuitError::HexDecoding(err)
//     }
// }

impl From<ark_crypto_primitives::Error> for GachaCircuitError {
    fn from(err: ark_crypto_primitives::Error) -> Self {
        GachaCircuitError::Primitive(err)
    }
}


// --- Conversion to JsValue (More Controlled String Formatting) ---

impl From<GachaCircuitError> for JsValue {
    fn from(error: GachaCircuitError) -> Self {
        let error_message = match error {
            // For variants wrapping external errors, create a descriptive string
            // Optionally include Debug formatting of the original error for more detail
            GachaCircuitError::ArkSerialization(ref err) => {
                format!("Error: Arkworks Serialization Failed (Details: {:?})", err)
            }
            GachaCircuitError::ArkSynthesis(ref err) => {
                format!("Error: Arkworks Synthesis Failed (Details: {:?})", err)
            }
             GachaCircuitError::HexDecoding(ref err) => {
                format!("Error: Hex Decoding Failed (Details: {:?})", err)
            }
             GachaCircuitError::Primitive(ref err) => {
                format!("Error: Cryptographic Primitive Failed (Details: {:?})", err)
            }
            // For variants already containing Strings or simple messages, use them directly
            _ => format!("{}", error), // Use the #[error("...")] message for others
        };
        JsValue::from_str(&error_message)
    }
}