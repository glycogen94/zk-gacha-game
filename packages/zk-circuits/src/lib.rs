use ark_bls12_381::Bls12_381; // SNARK Curve
use ark_groth16::{Groth16, Proof, ProvingKey, VerifyingKey};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_snark::SNARK;
use once_cell::sync::OnceCell;
use parking_lot::Mutex;
use serde_wasm_bindgen::from_value;
use utils::get_rng;
use wasm_bindgen::prelude::*;
use web_sys::console; // Import console for logging

// Import required modules
pub mod circuit;
pub mod error;
pub mod tests;
pub mod types;
pub mod utils;

// Re-export types needed for WASM boundary
pub use error::GachaCircuitError;
pub use types::WasmGachaCircuitInputs;

// Use concrete types defined in types.rs
use crate::circuit::UserPullCircuit;
use crate::types::{
    fr_from_hex, prepare_groth16_public_inputs, NativeGachaCircuitInputs, NativePoseidonConfig,
};

// --- Global Static Variables (Types updated) ---
static GACHA_PK: OnceCell<Mutex<ProvingKey<Bls12_381>>> = OnceCell::new();
static GACHA_VK: OnceCell<Mutex<VerifyingKey<Bls12_381>>> = OnceCell::new();
static POSEIDON_PARAMS: OnceCell<NativePoseidonConfig> = OnceCell::new(); // Uses NativePoseidonConfig

/// Sets up a panic hook
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    Ok(())
}

/// Initializes keys and parameters (Type signatures updated)
#[wasm_bindgen]
pub fn init_gacha_keys(
    pk_bytes: &[u8],
    vk_bytes: &[u8],
    params_bytes: &[u8],
) -> Result<(), JsValue> {
    if GACHA_PK.get().is_some() || GACHA_VK.get().is_some() || POSEIDON_PARAMS.get().is_some() {
        return Err(GachaCircuitError::SetupError("Already initialized".to_string()).into());
    }

    let params = NativePoseidonConfig::deserialize_compressed_unchecked(params_bytes)
        .map_err(|e| GachaCircuitError::Deserialization(format!("Params: {}", e)))?;
    let pk = ProvingKey::<Bls12_381>::deserialize_compressed_unchecked(pk_bytes)
        .map_err(|e| GachaCircuitError::Deserialization(format!("PK: {}", e)))?;
    let vk = VerifyingKey::<Bls12_381>::deserialize_compressed_unchecked(vk_bytes)
        .map_err(|e| GachaCircuitError::Deserialization(format!("VK: {}", e)))?;

    POSEIDON_PARAMS
        .set(params)
        .map_err(|_| GachaCircuitError::SetupError("Failed to set Params".to_string()))?;
    GACHA_PK
        .set(Mutex::new(pk))
        .map_err(|_| GachaCircuitError::SetupError("Failed to set PK".to_string()))?;
    GACHA_VK
        .set(Mutex::new(vk))
        .map_err(|_| GachaCircuitError::SetupError("Failed to set VK".to_string()))?;

    console::log_1(&"Gacha keys and params initialized successfully!".into());
    Ok(())
}

/// Generates proof (Type signatures updated)
#[wasm_bindgen]
pub fn generate_gacha_proof(inputs_js: JsValue) -> Result<Vec<u8>, JsValue> {
    // 1. Deserialize and Convert WASM inputs to Native inputs
    let wasm_inputs: WasmGachaCircuitInputs = from_value(inputs_js)
        .map_err(|e| GachaCircuitError::Deserialization(format!("WASM inputs: {}", e)))?;
    let native_inputs = NativeGachaCircuitInputs::try_from(wasm_inputs)?;

    // 2. Retrieve PK and Params
    let pk_lock = GACHA_PK.get().ok_or(GachaCircuitError::NotInitialized)?;
    let params = POSEIDON_PARAMS
        .get()
        .ok_or(GachaCircuitError::NotInitialized)?
        .clone(); // Clone needed params

    let pk = pk_lock.lock();

    // 3. Create the circuit instance (now concrete, no generics needed here)
    let circuit = UserPullCircuit::new(native_inputs, params);

    // 4. Generate the proof
    let mut rng = get_rng(None)
        .map_err(|e| GachaCircuitError::SetupError(format!("Failed to get RNG: {}", e)))?;

    let proof = Groth16::<Bls12_381>::prove(&pk, circuit, &mut rng)
        .map_err(|e| GachaCircuitError::ProofGeneration(e.to_string()))?;

    // 5. Serialize proof
    let mut proof_bytes = Vec::new();
    proof
        .serialize_compressed(&mut proof_bytes)
        .map_err(|e| GachaCircuitError::Serialization(format!("Proof serialization: {}", e)))?;

    Ok(proof_bytes)
}

/// Verifies proof (Type signatures updated)
#[wasm_bindgen]
pub fn verify_gacha_proof(merkle_root_hex: String, proof_bytes: &[u8]) -> Result<bool, JsValue> {
    // 1. Retrieve VK
    let vk_lock = GACHA_VK.get().ok_or(GachaCircuitError::NotInitialized)?;
    let vk = vk_lock.lock();

    // 2. Prepare public inputs (uses ConstraintField)
    let merkle_root = fr_from_hex(&merkle_root_hex)?;
    let public_inputs = prepare_groth16_public_inputs(merkle_root);

    // 3. Deserialize proof
    let proof = Proof::<Bls12_381>::deserialize_compressed_unchecked(proof_bytes)
        .map_err(|e| GachaCircuitError::Deserialization(format!("Proof: {}", e)))?;

    // 4. Verify using Groth16<Bls12_381>
    let is_valid = Groth16::<Bls12_381>::verify(&vk, &public_inputs, &proof)
        .map_err(|e| GachaCircuitError::ProofVerification(e.to_string()))?;

    Ok(is_valid)
}
