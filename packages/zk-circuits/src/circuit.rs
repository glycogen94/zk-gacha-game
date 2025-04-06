use ark_crypto_primitives::{
    crh::{
        poseidon::{
            constraints::{CRHGadget, TwoToOneCRHGadget}, // Import Poseidon gadgets
            CRH as PoseidonCRH,                          // Native Poseidon 2-to-1 CRH
        },
        CRHSchemeGadget,
    },
    merkle_tree::{
        constraints::{ConfigGadget, PathVar as MerklePathVar}, // Keep using the native trait alias for clarity
        IdentityDigestConverter,
    }, // Needed for GachaMerkleConfig definition
};
// Don't need CurveGroup or GG here anymore as Poseidon is field-based
use ark_r1cs_std::{
    fields::fp::FpVar, // Use FpVar for the constraint field
    prelude::*,
};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};

// Use types defined in types.rs
use crate::types::{
    ConstraintField, GachaMerkleConfig, NativeGachaCircuitInputs, NativeMerklePath,
    NativePoseidonConfig,
};

// --- Merkle Tree Config Gadget ---
// This configures the Merkle Tree operations *within the circuit*.
// It operates over the ConstraintField.
struct GachaMerkleConfigGadget;

impl ConfigGadget<GachaMerkleConfig, ConstraintField> for GachaMerkleConfigGadget {
    // Leaf data within the circuit is a slice of FpVar<ConstraintField>
    type Leaf = [FpVar<ConstraintField>];
    // Digests within the circuit are FpVar<ConstraintField>
    type LeafDigest = FpVar<ConstraintField>;
    type InnerDigest = FpVar<ConstraintField>;
    // Converter is identity since digests are already FpVar<ConstraintField>
    type LeafInnerConverter = IdentityDigestConverter<FpVar<ConstraintField>>;
    // Hash gadgets operate over ConstraintField
    type LeafHash = CRHGadget<ConstraintField>; // Gadget for H(secret_key_var, item_id_var)
    type TwoToOneHash = TwoToOneCRHGadget<ConstraintField>; // Gadget for H(left_hash_var, right_hash_var)
}

// Type alias for the Merkle Path Variable based on our circuit config
type GachaMerklePathVar =
    MerklePathVar<GachaMerkleConfig, ConstraintField, GachaMerkleConfigGadget>;

// Type alias for the Poseidon Parameter Variable
type PoseidonParametersVar = <CRHGadget<ConstraintField> as CRHSchemeGadget<
    PoseidonCRH<ConstraintField>,
    ConstraintField,
>>::ParametersVar;

/// ZK-SNARK circuit for verifying a gacha pull.
/// Defined over the ConstraintField (ark_bls12_381::Fr).
#[derive(Clone)]
pub struct UserPullCircuit {
    // --- Public Inputs ---
    pub merkle_root: ConstraintField,

    // --- Private Inputs (Witness) ---
    pub item_id: ConstraintField,
    pub secret_key: ConstraintField,
    // The native Merkle path structure containing native field elements
    pub native_merkle_path: NativeMerklePath,

    // --- Parameters (Constants) ---
    // Native Poseidon parameters for the ConstraintField
    pub poseidon_params: NativePoseidonConfig,
}

// Implementation does not need generics C, GG anymore
impl UserPullCircuit {
    /// Creates a new circuit instance from native inputs.
    pub fn new(inputs: NativeGachaCircuitInputs, poseidon_params: NativePoseidonConfig) -> Self {
        Self {
            merkle_root: inputs.merkle_root,
            item_id: inputs.item_id,
            secret_key: inputs.secret_key,
            native_merkle_path: inputs.native_merkle_path,
            poseidon_params,
        }
    }
}

// Implement ConstraintSynthesizer for the ConstraintField (ark_bls12_381::Fr)
impl ConstraintSynthesizer<ConstraintField> for UserPullCircuit {
    /// Generates the R1CS constraints for the circuit.
    fn generate_constraints(
        self,
        cs: ConstraintSystemRef<ConstraintField>,
    ) -> Result<(), SynthesisError> {
        // 1. Allocate Public Input Variables
        let merkle_root_var =
            FpVar::<ConstraintField>::new_input(cs.clone(), || Ok(self.merkle_root))?;

        // 2. Allocate Private Witness Variables
        let item_id_var = FpVar::<ConstraintField>::new_witness(cs.clone(), || Ok(self.item_id))?;
        let secret_key_var =
            FpVar::<ConstraintField>::new_witness(cs.clone(), || Ok(self.secret_key))?;

        // Allocate the Merkle path witness using the *native* path object.
        // This internally allocates the necessary FpVar and Boolean variables.
        let path_var = GachaMerklePathVar::new_witness(
            ark_relations::ns!(cs, "merkle_path_witness"),
            || Ok(self.native_merkle_path), // Provide the native path struct
        )?;

        // 3. Allocate Parameters as Constants
        // Allocate the Poseidon parameters as circuit constants.
        let params_var = PoseidonParametersVar::new_constant(cs.clone(), self.poseidon_params)?;
        // Since LeafHash and TwoToOneHash use the same params in our config:
        let leaf_params_var = &params_var;
        let two_to_one_params_var = &params_var;

        // --- Define Constraints ---

        // Constraint 1: Verify the Merkle path membership.
        // Define the leaf data *within the circuit* as required by GachaMerkleConfigGadget::Leaf
        let leaf_data_vars = &[secret_key_var, item_id_var];

        // Call verify_membership on the allocated path variable.
        // It will internally:
        // - Use LeafHash (CRHGadget) with leaf_params_var to hash leaf_data_vars.
        // - Use TwoToOneHash (TwoToOneCRHGadget) with two_to_one_params_var to hash inner nodes.
        // - Compare the calculated root with merkle_root_var.
        let membership_result = path_var.verify_membership(
            leaf_params_var,       // Params for Leaf Hash
            two_to_one_params_var, // Params for Inner Hash
            &merkle_root_var,      // Public Root to verify against
            leaf_data_vars,        // Leaf data *before* hashing
        )?;

        // Enforce that the membership proof verification succeeded.
        membership_result.enforce_equal(&Boolean::TRUE)?;

        Ok(())
    }
}
