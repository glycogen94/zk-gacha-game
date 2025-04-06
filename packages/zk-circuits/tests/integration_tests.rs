//! Integration tests for the zk_circuits package.
//! These tests run natively using `cargo test`.

use ark_bls12_381::{Bls12_381, Fr}; // Use Fr directly as ConstraintField
use ark_crypto_primitives::crh::CRHScheme;
use ark_crypto_primitives::{
    crh::poseidon::CRH as PoseidonCRH,
    merkle_tree::MerkleTree, // Native Merkle Tree
    sponge::poseidon::{find_poseidon_ark_and_mds, PoseidonConfig},
};
use ark_ff::{PrimeField, UniformRand};
use ark_groth16::{Groth16, PreparedVerifyingKey, ProvingKey, VerifyingKey};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystem};
use ark_snark::SNARK;
use ark_std::rand::RngCore;
use ark_std::rand::SeedableRng;
use ark_std::{test_rng, vec::Vec};

// Import types and circuit from the library crate
use zk_circuits::{
    circuit::UserPullCircuit, // Optional: Use if testing error conditions
    types::{
        prepare_groth16_public_inputs, ConstraintField, GachaMerkleConfig,
        NativeGachaCircuitInputs, NativePoseidonConfig,
    },
};

// --- Helper Functions for Tests ---

/// Generates default Poseidon parameters for testing.
fn setup_poseidon_params() -> NativePoseidonConfig {
    // Use standard configuration for Poseidon with rate=2
    let rate = 2;
    let capacity = 1;
    let full_rounds = 8;
    // Using 31 based on reference benchmarks, adjust if needed for security level
    let partial_rounds = 31;
    let alpha = 17; // Common choice for BLS12-381 Fr

    let (ark, mds) = find_poseidon_ark_and_mds::<ConstraintField>(
        ConstraintField::MODULUS_BIT_SIZE as u64,
        rate + capacity,
        full_rounds as u64,
        partial_rounds as u64,
        0, // skip matrices
    );

    PoseidonConfig {
        full_rounds,
        partial_rounds,
        alpha: alpha as u64,
        ark,
        mds,
        rate,
        capacity,
    }
}

/// Creates valid test data including inputs and the Merkle tree components.
/// Returns (circuit_inputs, poseidon_params, merkle_tree_root)
fn create_test_data(
    poseidon_params: &NativePoseidonConfig,
    tree_size: usize, // Must be power of 2
    target_leaf_index: usize,
) -> Result<(NativeGachaCircuitInputs, Fr), Box<dyn std::error::Error>> {
    assert!(tree_size.is_power_of_two());
    assert!(target_leaf_index < tree_size);

    let mut rng = test_rng();

    // 1. Generate data for the target leaf
    let target_item_id = Fr::rand(&mut rng);
    let target_secret_key = Fr::rand(&mut rng);
    let target_leaf_hash = PoseidonCRH::<ConstraintField>::evaluate(
        poseidon_params,
        [target_secret_key, target_item_id],
    )?;

    // 2. Generate data for other leaves
    let mut leaf_hashes = Vec::with_capacity(tree_size);
    for i in 0..tree_size {
        if i == target_leaf_index {
            leaf_hashes.push(target_leaf_hash);
        } else {
            // Other leaves can have dummy data for this test
            let dummy_item_id = Fr::rand(&mut rng);
            let dummy_secret_key = Fr::rand(&mut rng);
            let leaf_hash = PoseidonCRH::<ConstraintField>::evaluate(
                poseidon_params,
                [dummy_secret_key, dummy_item_id],
            )?;
            leaf_hashes.push(leaf_hash);
        }
    }

    // 3. Build the Merkle Tree
    let merkle_tree = MerkleTree::<GachaMerkleConfig>::new_with_leaf_digest(
        poseidon_params, // leaf params
        poseidon_params, // two_to_one params
        leaf_hashes,
    )?;
    let merkle_root = merkle_tree.root();

    // 4. Generate the proof for the target leaf
    let native_merkle_path = merkle_tree.generate_proof(target_leaf_index)?;

    // 5. Create NativeGachaCircuitInputs
    let circuit_inputs = NativeGachaCircuitInputs {
        merkle_root, // This will be the public input
        item_id: target_item_id,
        secret_key: target_secret_key,
        native_merkle_path,
    };

    Ok((circuit_inputs, merkle_root))
}

/// Sets up Groth16 keys for a given circuit instance.
/// This is slower than loading pre-generated keys but useful for testing.
fn setup_groth16_keys(
    poseidon_params: &NativePoseidonConfig,
    tree_size: usize, // Must be power of 2
    target_leaf_index: usize,
) -> Result<
    (
        ProvingKey<Bls12_381>,
        VerifyingKey<Bls12_381>,
        PreparedVerifyingKey<Bls12_381>,
    ),
    Box<dyn std::error::Error>,
> {
    let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(ark_std::test_rng().next_u64());
    // Create a dummy circuit instance just for setup
    let (dummy_inputs, _) = create_test_data(poseidon_params, tree_size, target_leaf_index)?; // Small tree for setup
    let dummy_circuit = UserPullCircuit::new(dummy_inputs, poseidon_params.clone());

    let (pk, vk) = Groth16::<Bls12_381>::circuit_specific_setup(dummy_circuit, &mut rng)?;
    let pvk = Groth16::<Bls12_381>::process_vk(&vk)?;
    Ok((pk, vk, pvk))
}

// --- Test Cases ---

#[test]
fn test_circuit_constraints_satisfied() {
    let params = setup_poseidon_params();
    let tree_size = 8; // Small power of 2 for testing
    let target_leaf_index = 3;

    // Create valid data and path
    let result = create_test_data(&params, tree_size, target_leaf_index);
    assert!(result.is_ok(), "Failed to create test data");
    let (inputs, _merkle_root) = result.unwrap();

    // Create the circuit instance with the valid witness
    let circuit = UserPullCircuit::new(inputs, params);

    // Check constraints
    let cs = ConstraintSystem::<ConstraintField>::new_ref();
    circuit
        .generate_constraints(cs.clone())
        .expect("Constraint generation failed");

    assert!(
        cs.is_satisfied()
            .expect("Failed to check constraint satisfaction"),
        "Circuit constraints not satisfied"
    );
    println!(
        "Number of constraints for membership proof ({} leaves): {}",
        tree_size,
        cs.num_constraints()
    );
}

#[test]
fn test_circuit_constraints_unsatisfied() {
    let params = setup_poseidon_params();
    let tree_size = 8;
    let target_leaf_index = 3;
    let mut rng = test_rng();

    // Create valid data initially
    let result = create_test_data(&params, tree_size, target_leaf_index);
    assert!(result.is_ok());
    let (mut inputs, _merkle_root) = result.unwrap();

    // --- Introduce an inconsistency ---
    // Change the secret key without updating the path/root
    inputs.secret_key = Fr::rand(&mut rng);

    // Create the circuit instance with the invalid witness
    let circuit = UserPullCircuit::new(inputs, params);

    // Check constraints
    let cs = ConstraintSystem::<ConstraintField>::new_ref();
    circuit
        .generate_constraints(cs.clone())
        .expect("Constraint generation should succeed even with bad witness");

    // Constraint satisfaction check should FAIL
    assert!(
        !cs.is_satisfied()
            .expect("Failed to check constraint satisfaction"),
        "Circuit constraints should NOT be satisfied with inconsistent witness"
    );
}

#[test]
fn test_proof_generation_and_verification() {
    let params = setup_poseidon_params();
    let tree_size = 16; // Slightly larger tree
    let target_leaf_index = 7;

    // 1. Setup keys
    let key_setup_result = setup_groth16_keys(&params, tree_size, target_leaf_index);
    assert!(key_setup_result.is_ok(), "Failed to setup Groth16 keys");
    let (pk, _vk, pvk) = key_setup_result.unwrap();

    // 2. Create valid test data
    let data_creation_result = create_test_data(&params, tree_size, target_leaf_index);
    assert!(data_creation_result.is_ok(), "Failed to create test data");
    let (inputs, merkle_root) = data_creation_result.unwrap();
    let native_verify_ok = inputs
        .native_merkle_path
        .verify(
            &params, // leaf params
            &params, // two_to_one params
            &merkle_root,
            [inputs.secret_key, inputs.item_id], // Original leaf data
        )
        .expect("Native path verification failed");
    assert!(
        native_verify_ok,
        "Native Merkle path verification failed BEFORE proving"
    );

    // 3. Create the circuit instance
    let circuit = UserPullCircuit::new(inputs, params);

    // 4. Generate proof
    let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(ark_std::test_rng().next_u64());
    let proof_result = Groth16::<Bls12_381>::prove(&pk, circuit, &mut rng);
    assert!(proof_result.is_ok(), "Proof generation failed");
    let proof = proof_result.unwrap();

    // 5. Prepare public inputs
    let public_inputs = prepare_groth16_public_inputs(merkle_root);

    // 6. Verify the proof
    let verification_result = Groth16::<Bls12_381>::verify_with_processed_vk(
        &pvk,
        &public_inputs, // Public inputs should be &[Fr]
        &proof,
    );
    assert!(
        verification_result.is_ok(),
        "Verification check returned error"
    );
    assert!(verification_result.unwrap(), "Proof verification failed");
}

#[test]
fn test_invalid_proof_wrong_public_input() {
    let params = setup_poseidon_params();
    let tree_size = 16;
    let target_leaf_index = 7;
    let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(ark_std::test_rng().next_u64());

    // 1. Setup keys
    let key_setup_result = setup_groth16_keys(&params, tree_size, target_leaf_index);
    assert!(key_setup_result.is_ok());
    let (pk, _vk, pvk) = key_setup_result.unwrap();

    // 2. Create valid test data
    let data_creation_result = create_test_data(&params, tree_size, target_leaf_index);
    assert!(data_creation_result.is_ok());
    let (inputs, _correct_merkle_root) = data_creation_result.unwrap();

    // 3. Create the circuit instance
    let circuit = UserPullCircuit::new(inputs, params);

    // 4. Generate a *valid* proof
    let proof_result = Groth16::<Bls12_381>::prove(&pk, circuit, &mut rng);
    assert!(proof_result.is_ok());
    let proof = proof_result.unwrap();

    // 5. Prepare *incorrect* public inputs
    let incorrect_merkle_root = Fr::rand(&mut rng); // Use a random root
    let incorrect_public_inputs = prepare_groth16_public_inputs(incorrect_merkle_root);

    // 6. Verify the proof against incorrect inputs
    let verification_result =
        Groth16::<Bls12_381>::verify_with_processed_vk(&pvk, &incorrect_public_inputs, &proof);
    assert!(
        verification_result.is_ok(),
        "Verification check returned error unexpectedly"
    );
    // Verification should FAIL
    assert!(
        !verification_result.unwrap(),
        "Proof verification succeeded with incorrect public input"
    );
}
