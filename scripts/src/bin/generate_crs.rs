// scripts/src/bin/generate_crs.rs

use ark_bls12_381::Bls12_381;
use ark_crypto_primitives::sponge::poseidon::{find_poseidon_ark_and_mds, PoseidonConfig};
use ark_ff::{PrimeField, UniformRand};
use ark_groth16::{Groth16, ProvingKey, VerifyingKey};
use ark_serialize::CanonicalSerialize;
use ark_snark::SNARK;
use ark_std::{rand::{RngCore, SeedableRng}, test_rng};
use std::{fs::File, path::PathBuf};
use ark_std::Zero;

// Use the config module from the scripts crate OR declare locally if no lib.rs
use zk_gacha_scripts::config::TREE_HEIGHT;
// mod config; use config::TREE_HEIGHT; // Alternative if no lib.rs

// Import from the actual zk_circuits library
use zk_circuits::{
    circuit::UserPullCircuit,
    types::{ConstraintField, NativeGachaCircuitInputs, NativeMerklePath, NativePoseidonConfig},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating CRS for ZK Gacha Game (Tree Height: {})...", TREE_HEIGHT);

    let output_dir = PathBuf::from("./output");
    std::fs::create_dir_all(&output_dir)?;

    println!("Generating Poseidon parameters...");
    let poseidon_params = generate_poseidon_parameters()?;

    println!("Creating dummy circuit...");
    let dummy_circuit = create_dummy_circuit(poseidon_params.clone())?;

    println!("Generating Groth16 keys...");
    let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(test_rng().next_u64());
    let (pk, vk) = Groth16::<Bls12_381>::circuit_specific_setup(dummy_circuit, &mut rng)?;

    println!("Saving parameters and keys...");
    save_params(&poseidon_params, &output_dir.join("params.bin"))?;
    save_pk(&pk, &output_dir.join("gacha_pk.bin"))?;
    save_vk(&vk, &output_dir.join("gacha_vk.bin"))?;

    println!( "CRS generation complete! Files saved to {}", output_dir.display() );
    Ok(())
}

fn generate_poseidon_parameters() -> Result<NativePoseidonConfig, Box<dyn std::error::Error>> {
    let rate = 2;
    let capacity = 1;
    let full_rounds = 8;
    let partial_rounds = 31;
    let alpha = 17;

    let (ark, mds) = find_poseidon_ark_and_mds::<ConstraintField>(
        ConstraintField::MODULUS_BIT_SIZE as u64,
        rate + capacity,
        full_rounds as u64,
        partial_rounds as u64,
        0,
    );

    Ok(PoseidonConfig { full_rounds, partial_rounds, alpha: alpha as u64, ark, mds, rate, capacity })
}

fn create_dummy_circuit(
    poseidon_params: NativePoseidonConfig,
) -> Result<UserPullCircuit, Box<dyn std::error::Error>> {
    let mut rng = test_rng();

    let merkle_root = ConstraintField::rand(&mut rng);
    let item_id = ConstraintField::rand(&mut rng);
    let secret_key = ConstraintField::rand(&mut rng);

    // Construct dummy path based on TREE_HEIGHT from config
    let tree_height = TREE_HEIGHT;
    let leaf_index = 0;
    // Verify path length expected by NativeMerklePath / circuit gadget
    let auth_path_len = tree_height; // Assuming path includes sibling upwards
    let auth_path_vec: Vec<ConstraintField> = (0..auth_path_len)
        .map(|_| ConstraintField::rand(&mut rng))
        .collect();
    let leaf_sibling_hash = ConstraintField::rand(&mut rng);

    // Adjust construction based on NativeMerklePath definition in zk_circuits::types
    let native_merkle_path = NativeMerklePath {
        leaf_index,
        auth_path: auth_path_vec,
        leaf_sibling_hash, // Placeholder for leaf sibling hash

        // Adjust fields if necessary
    };

    let inputs = NativeGachaCircuitInputs {
        merkle_root,
        item_id,
        secret_key,
        native_merkle_path,
    };

    Ok(UserPullCircuit::new(inputs, poseidon_params))
}

// --- Save Functions ---
fn save_params( params: &NativePoseidonConfig, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(path)?;
    params.serialize_compressed(&mut file)?;
    Ok(())
}

fn save_pk( pk: &ProvingKey<Bls12_381>, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(path)?;
    pk.serialize_compressed(&mut file)?;
    Ok(())
}

fn save_vk( vk: &VerifyingKey<Bls12_381>, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(path)?;
    vk.serialize_compressed(&mut file)?;
    Ok(())
}