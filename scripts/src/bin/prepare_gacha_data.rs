// scripts/src/bin/prepare_gacha_data.rs

use ark_crypto_primitives::merkle_tree::{Config as MerkleConfig, MerkleTree};
use ark_ff::UniformRand;
use ark_serialize::CanonicalDeserialize;
use ark_std::{test_rng, vec::Vec, Zero};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{create_dir_all, File},
    io::{Read, Write},
    path::PathBuf,
};
use ark_crypto_primitives::{
    // CRH는 LeafHash 타입으로 필요, TwoToOneCRH는 리프 해시 계산 및 InnerHash 타입으로 필요
    crh::{poseidon::{CRH as PoseidonCRH},  TwoToOneCRHScheme},
};
use ark_crypto_primitives::crh::CRHScheme;
use ark_crypto_primitives::merkle_tree::Path;
// Use the config module
use zk_gacha_scripts::config::{
    define_gacha_items, ItemDetails, ItemMaster, TREE_HEIGHT, TREE_SIZE,
};
// OR: mod config; use config::*;

// Import from zk_circuits library
use zk_circuits::types::{
    fr_to_hex, ConstraintField, GachaMerkleConfig, NativePoseidonConfig,
};

// --- Structs ---
#[derive(Serialize, Deserialize, Debug)]
struct ItemProofData {
    #[serde(rename = "itemIdHex")]
    item_id_hex: String,
    #[serde(rename = "secretKeyHex")]
    secret_key_hex: String,
    #[serde(rename = "merklePathNodesHex")]
    merkle_path_nodes_hex: Vec<String>,
    #[serde(rename = "leafSiblingHashHex")]
    leaf_sibling_hash_hex: String,
    #[serde(rename = "leafIndex")]
    leaf_index: usize,
}

struct GeneratedLeafData {
    item_id: ConstraintField,
    secret_key: ConstraintField,
    leaf_digest: ConstraintField, // Leaf pair [secret_key, item_id]
    item_id_hex: String,
    secret_key_hex: String,
}

// --- Main Logic ---
fn main() -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "Preparing Gacha Game data (Tree Size: {}, Height: {})...",
            TREE_SIZE, TREE_HEIGHT
        );
    
        let output_dir = PathBuf::from("./output");
        let items_dir = output_dir.join("items");
        create_dir_all(&items_dir)?;
    
        let item_master_list = define_gacha_items();
        let num_items = item_master_list.len();
        let tree_size = TREE_SIZE;
        assert!(num_items <= tree_size);
        println!("Padding item list from {} to {}.", num_items, tree_size);
    
        let params_path = output_dir.join("params.bin");
        println!("Loading Poseidon parameters from: {:?}", params_path);
        let poseidon_params = load_poseidon_params(&params_path)?;
    
        // 4. Generate leaf data AND pre-calculate digests
        println!("Generating leaf data and calculating leaf digests...");
        let generated_leaf_data =
            generate_leaf_data_and_digests(&item_master_list, tree_size, &poseidon_params)?;
    
        // Get the pre-calculated digests for the tree
        let leaf_digests: Vec<ConstraintField> = generated_leaf_data
            .iter()
            .map(|d| d.leaf_digest) // Use the stored digest
            .collect();
        println!("Generated {} leaf digests.", leaf_digests.len());
    
        // 5. Build the Merkle tree using PRE-HASHED digests
        println!("Building Merkle tree with leaf digests...");
        // Use new_with_leaf_digest, providing inner hash params and the digests
        let merkle_tree = MerkleTree::<GachaMerkleConfig>::new_with_leaf_digest(
            &poseidon_params, // Params for LeafHash (CRH<ConstraintField>)
            &poseidon_params, // Params for TwoToOneHash (TwoToOneCRH<ConstraintField>)
            leaf_digests,    // Pass the Vec<ConstraintField> of digests
                )?;
        println!("Merkle tree built successfully.");
    
        // 6. Save Merkle root
        let merkle_root = merkle_tree.root();
        let merkle_root_hex = fr_to_hex(&merkle_root)?;
        let root_path = output_dir.join("merkle_root.hex");
        save_merkle_root(&merkle_root_hex, &root_path)?;
        println!("Merkle Root: {}", merkle_root_hex);
    
        // 7. Generate, verify, and save proof data for each leaf
        println!("Generating, VERIFYING, and saving Merkle proofs...");
        let mut key_list = Vec::new();
        for i in 0..tree_size {
            let leaf_data = &generated_leaf_data[i]; // Contains original keys/ids AND digest
    
            // Generate proof using index i. Path contains digests.
            let proof = merkle_tree.generate_proof(i)?;
            println!("MERKLE_PROOF for leaf {}:", i);
            println!("  leaf_index: {}", proof.leaf_index);
    
            // --- Verify the generated path ---
            // Verify needs the ORIGINAL leaf data ([secret, id]) to re-calculate the hash.
            let original_leaf_pair = [leaf_data.secret_key, leaf_data.item_id];
            let is_valid_path = proof.verify(
                &poseidon_params, // Params for LeafHash (must match GachaMerkleConfig::LeafHash type's needs)
                &poseidon_params, // Params for TwoToOneHash
                &merkle_root,
                original_leaf_pair, // Pass the original [secret, id] pair to verify against
            )?;
    
            if !is_valid_path {
                return Err(format!("CRITICAL ERROR: Merkle proof verification failed for leaf index {}!", i).into());
            }
            println!("  Proof VERIFIED successfully against generated root and original leaf data.");
            // --- End Verification ---
    
    
            // --- Path Extraction --- (Same logic as before, path contains digests)
            let (leaf_sibling_hash, merkle_path_nodes) =
                if tree_size == 1 || proof.auth_path.is_empty() {
                    (ConstraintField::zero(), Vec::new())
                } else {
                    let sibling = proof.auth_path[0]; // Sibling is a digest
                    let path_nodes = proof.auth_path.iter().skip(1).cloned().collect::<Vec<_>>(); // Path nodes are digests
                    (sibling, path_nodes)
                };
    
            let leaf_sibling_hash_hex = fr_to_hex(&leaf_sibling_hash)?;
            let merkle_path_nodes_hex: Vec<String> = merkle_path_nodes
                .iter()
                .map(|h| fr_to_hex(h).expect("Hex conversion failed for path node"))
                .collect();
            // ... (logging extracted path info) ...
            println!("  leaf_sibling_hash: {}", leaf_sibling_hash_hex);
            println!("  extracted path_nodes count (for JSON): {}", merkle_path_nodes_hex.len());
    
    
            // Create the output struct (still need original keys/ids for prover)
            let item_proof_output = ItemProofData {
                item_id_hex: leaf_data.item_id_hex.clone(),
                secret_key_hex: leaf_data.secret_key_hex.clone(),
                merkle_path_nodes_hex,
                leaf_sibling_hash_hex,
                leaf_index: proof.leaf_index,
            };
    
            // ... (log writing data) ...
            println!("WRITING ITEM_PROOF DATA for item {}:", i);
            // ... log fields ...
    
            // Save file
            let item_file_name = format!("item_{}.json", i);
            let item_path = items_dir.join(&item_file_name);
            save_item_proof_data(&item_proof_output, &item_path)?;
            key_list.push(format!("items/{}", item_file_name));
        } // End loop
    

    let key_list_path = output_dir.join("key_list.txt");
    save_key_list(&key_list, &key_list_path)?;
    println!("Key list saved to: {:?}", key_list_path);

    let mut item_master_map = HashMap::new();
     for i in 0..item_master_list.len() {
         if let Some(leaf_info) = generated_leaf_data.get(i) {
              let master_item = &item_master_list[i];
              let item_details = ItemDetails {
                  id: master_item.id.clone(),
                  name: master_item.name.clone(),
                  imageUrl: master_item.image_url.clone(),
                  rarity: master_item.rarity.clone(),
              };
             item_master_map.insert(leaf_info.item_id_hex.clone(), item_details);
         }
     }
    let master_path = output_dir.join("item_master.json");
    save_item_master_map(&item_master_map, &master_path)?;
    println!("Item master map saved to: {:?}", master_path);

    println!("Data preparation complete!");
    Ok(())
}

// --- Helper Functions ---
fn load_poseidon_params( params_path: &PathBuf ) -> Result<NativePoseidonConfig, Box<dyn std::error::Error>> {
    let mut file = File::open(params_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(NativePoseidonConfig::deserialize_compressed_unchecked(&buffer[..])?)
}

fn generate_leaf_data_and_digests( item_master_list: &[ItemMaster], tree_size: usize, poseidon_params: &NativePoseidonConfig ) -> Result<Vec<GeneratedLeafData>, Box<dyn std::error::Error>> {
    let mut rng = test_rng();
    let mut generated_data = Vec::with_capacity(tree_size);
    let num_master_items = item_master_list.len();

    for i in 0..tree_size {
        let item_id_num = i as u64;
        let item_id = ConstraintField::rand(&mut rng);
        let secret_key = ConstraintField::rand(&mut rng);

        // Calculate the leaf digest using TwoToOneCRH for the pair
        let leaf_digest = PoseidonCRH::<ConstraintField>::evaluate(
            poseidon_params,
            [secret_key, item_id],
        )?;
        
        let item_id_hex = fr_to_hex(&item_id)?;
        let secret_key_hex = fr_to_hex(&secret_key)?;
        let leaf_digest_hex = fr_to_hex(&leaf_digest)?;

        println!("LEAF_DATA {}", i);
        // ... (log id, secret) ...
        println!("  Leaf Digest: {}", leaf_digest_hex);

        generated_data.push(GeneratedLeafData {
            item_id,
            secret_key,
            leaf_digest,
            item_id_hex,
            secret_key_hex,
        });
    }
    Ok(generated_data)
}

fn save_merkle_root( merkle_root_hex: &str, path: &PathBuf ) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(path)?;
    file.write_all(merkle_root_hex.as_bytes())?;
    Ok(())
}

fn save_item_proof_data( item_output: &ItemProofData, path: &PathBuf ) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(item_output)?;
    let mut file = File::create(path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

fn save_key_list( key_list: &[String], path: &PathBuf ) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(path)?;
    for key in key_list { writeln!(file, "{}", key)?; }
    Ok(())
}

fn save_item_master_map( item_map: &HashMap<String, ItemDetails>, path: &PathBuf ) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(item_map)?;
    let mut file = File::create(path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}