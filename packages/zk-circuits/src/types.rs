use crate::error::GachaCircuitError;
use ark_crypto_primitives::sponge::poseidon::PoseidonConfig;
use ark_crypto_primitives::{
    crh::poseidon,
    merkle_tree::{Config as MerkleConfig, IdentityDigestConverter, Path as MerklePath},
};
use ark_ff::BigInteger;
use ark_ff::PrimeField;
use ark_std::vec::Vec;
use serde::{Deserialize, Serialize};

// --- Field Definitions ---
// This is the field over which the R1CS constraints are defined.
// It's the scalar field of the BLS12-381 curve used for Groth16.
// It's also the field Poseidon will operate on within the circuit.
pub type ConstraintField = ark_bls12_381::Fr;

// --- Native Merkle Tree Configuration ---
// This configures the native Rust Merkle Tree operations.
#[derive(Debug, Clone)]
pub struct GachaMerkleConfig;

impl MerkleConfig for GachaMerkleConfig {
    // Native leaf data: A slice of ConstraintField elements (e.g., [secret_key, item_id])
    type Leaf = [ConstraintField];
    // Native digest types are ConstraintField elements
    type LeafDigest = ConstraintField;
    type InnerDigest = ConstraintField;
    // Converter is direct identity since digests are already ConstraintField elements
    type LeafInnerDigestConverter = IdentityDigestConverter<ConstraintField>;
    // Hash functions operate over ConstraintField
    type LeafHash = poseidon::CRH<ConstraintField>;
    type TwoToOneHash = poseidon::TwoToOneCRH<ConstraintField>;
}

// --- Alias for Native Merkle Path ---
// This path uses the native digest types (ConstraintField)
pub type NativeMerklePath = MerklePath<GachaMerkleConfig>;

// --- Core Native Types ---
// Configuration for the native Poseidon hash function
pub type NativePoseidonConfig = PoseidonConfig<ConstraintField>;

// --- WASM Data Transfer Object ---
// (No changes needed here, hex strings are field-agnostic at this level)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WasmGachaCircuitInputs {
    #[serde(rename = "merkleRoot")]
    pub merkle_root_hex: String,
    #[serde(rename = "itemIdHex")]
    pub item_id_hex: String,
    #[serde(rename = "secretKeyHex")]
    pub secret_key_hex: String,
    #[serde(rename = "merklePathNodesHex")]
    pub merkle_path_nodes_hex: Vec<String>,
    #[serde(rename = "leafSiblingHashHex")]
    pub leaf_sibling_hash_hex: String,
    #[serde(rename = "leafIndex")]
    pub leaf_index: usize,
}

// --- Native Rust Input Structure (for circuit construction) ---
// Holds native ConstraintField elements ready for the circuit
#[derive(Clone, Debug)]
pub struct NativeGachaCircuitInputs {
    // Public Input
    pub merkle_root: ConstraintField,
    // Private Witness
    pub item_id: ConstraintField,
    pub secret_key: ConstraintField,
    pub native_merkle_path: NativeMerklePath,
}

// --- Conversion Functions ---

/// Converts a hex string to ConstraintField.
pub fn fr_from_hex(hex_str: &str) -> Result<ConstraintField, GachaCircuitError> {
    let stripped = hex_str.trim_start_matches("0x");
    let bytes = hex::decode(stripped)?;
    Ok(ConstraintField::from_le_bytes_mod_order(&bytes))
}

/// Converts ConstraintField to a hex string.
pub fn fr_to_hex(fr: &ConstraintField) -> Result<String, GachaCircuitError> {
    let bytes = fr.into_bigint().to_bytes_le(); // Use BigInt bytes for standard hex rep
    Ok(format!("0x{}", hex::encode(&bytes)))
}

/// Converts WASM DTO to Native Rust structure.
impl TryFrom<WasmGachaCircuitInputs> for NativeGachaCircuitInputs {
    type Error = GachaCircuitError;

    fn try_from(wasm_inputs: WasmGachaCircuitInputs) -> Result<Self, Self::Error> {
        let merkle_root = fr_from_hex(&wasm_inputs.merkle_root_hex)?;
        let item_id = fr_from_hex(&wasm_inputs.item_id_hex)?;
        let secret_key = fr_from_hex(&wasm_inputs.secret_key_hex)?;
        let leaf_sibling_hash = fr_from_hex(&wasm_inputs.leaf_sibling_hash_hex)?;

        let auth_path_nodes = wasm_inputs
            .merkle_path_nodes_hex
            .iter()
            .map(|args| fr_from_hex(args))
            .collect::<Result<Vec<_>, _>>()?;

        let native_merkle_path = NativeMerklePath {
            leaf_index: wasm_inputs.leaf_index,
            auth_path: auth_path_nodes,
            leaf_sibling_hash,
        };

        Ok(NativeGachaCircuitInputs {
            merkle_root,
            item_id,
            secret_key,
            native_merkle_path,
        })
    }
}

/// Prepares the public inputs vector (ConstraintField elements) for Groth16 verification.
pub fn prepare_groth16_public_inputs(merkle_root: ConstraintField) -> Vec<ConstraintField> {
    vec![merkle_root]
}
