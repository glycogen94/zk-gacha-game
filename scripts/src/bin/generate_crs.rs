use ark_bls12_381::Bls12_381; // Use BLS12-381
use ark_crypto_primitives::sponge::poseidon::{
    find_poseidon_ark_and_mds, PoseidonConfig,
};
use ark_ff::{PrimeField, UniformRand}; // Import PrimeField for MODULUS_BIT_SIZE
use ark_groth16::{Groth16, ProvingKey, VerifyingKey};
use ark_serialize::CanonicalSerialize;
use ark_snark::SNARK;
use std::{fs::File, path::PathBuf};
use ark_std::rand::SeedableRng;
use ark_std::rand::RngCore;

// Import from our zk_circuits package
use zk_circuits::{
    circuit::UserPullCircuit,
    types::{ConstraintField, NativeGachaCircuitInputs, NativeMerklePath, NativePoseidonConfig}, // Use updated types
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating CRS (Common Reference String) for ZK Gacha Game (BLS12-381)...");

    // 1. Create output directory
    let output_dir = PathBuf::from("./output");
    std::fs::create_dir_all(&output_dir)?;

    // 2. Generate Poseidon parameters
    println!("Generating Poseidon parameters...");
    let poseidon_params = generate_poseidon_parameters()?;

    // 3. Generate a dummy circuit
    println!("Creating dummy circuit for CRS generation...");
    let dummy_circuit = create_dummy_circuit(poseidon_params.clone())?;

    // 4. Generate Groth16 keys
    println!("Generating Groth16 proving and verifying keys...");
    let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(ark_std::test_rng().next_u64());

    // Note: Use Bls12_381 here
    let (pk, vk) = Groth16::<Bls12_381>::circuit_specific_setup(dummy_circuit, &mut rng)?;

    // 5. Save files
    save_params(&poseidon_params, &output_dir.join("params.bin"))?;
    save_pk(&pk, &output_dir.join("gacha_pk.bin"))?;
    save_vk(&vk, &output_dir.join("gacha_vk.bin"))?;

    println!(
        "CRS generation complete! Files saved to {}",
        output_dir.display()
    );
    Ok(())
}

/// Generate Poseidon parameters (using defaults or custom logic)
fn generate_poseidon_parameters() -> Result<NativePoseidonConfig, Box<dyn std::error::Error>> {
    // Use standard configuration for Poseidon with rate=2 (for [secret, item_id])
    let rate = 2;
    let capacity = 1;
    let full_rounds = 8;
    let partial_rounds = 31; // Check recommended values for security
    let alpha = 17; // Common choice for BLS12-381 Fr

    let (ark, mds) = find_poseidon_ark_and_mds::<ConstraintField>(
        ConstraintField::MODULUS_BIT_SIZE as u64,
        rate + capacity, // t = rate + capacity
        full_rounds as u64,
        partial_rounds as u64,
        0, // skip matrices parameter (use 0 for default generation)
    );

    let poseidon_config = PoseidonConfig {
        full_rounds,
        partial_rounds,
        alpha: alpha as u64,
        ark,
        mds,
        rate,
        capacity,
    };
    Ok(poseidon_config)
}

/// Create a dummy circuit instance for setup
fn create_dummy_circuit(
    poseidon_params: NativePoseidonConfig,
) -> Result<UserPullCircuit, Box<dyn std::error::Error>> {
    let mut rng = ark_std::test_rng();

    // Dummy values (actual content doesn't matter for setup)
    let merkle_root = ConstraintField::rand(&mut rng);
    let item_id = ConstraintField::rand(&mut rng);
    let secret_key = ConstraintField::rand(&mut rng);

    // Dummy path for a tree height (e.g., 10 for 1024 leaves)
    let tree_height = 10; // Adjust if needed
    let leaf_index = 0; // Example index
    let auth_path = (0..(tree_height - 2))
        .map(|_| ConstraintField::rand(&mut rng))
        .collect();
    let leaf_sibling_hash = ConstraintField::rand(&mut rng);

    let native_merkle_path = NativeMerklePath {
        leaf_index,
        auth_path,
        leaf_sibling_hash,
    };

    let inputs = NativeGachaCircuitInputs {
        merkle_root,
        item_id,
        secret_key,
        native_merkle_path,
    };

    Ok(UserPullCircuit::new(inputs, poseidon_params))
}

// --- Save Functions (Unchanged but ensure types match) ---
fn save_params(
    params: &NativePoseidonConfig,
    path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(path)?;
    // Use compressed serialization for efficiency
    params.serialize_compressed(&mut file)?;
    Ok(())
}

fn save_pk(
    pk: &ProvingKey<Bls12_381>,
    path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(path)?;
    pk.serialize_compressed(&mut file)?;
    Ok(())
}

fn save_vk(
    vk: &VerifyingKey<Bls12_381>,
    path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(path)?;
    vk.serialize_compressed(&mut file)?;
    Ok(())
}
