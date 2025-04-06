use ark_crypto_primitives::{
    crh::{poseidon::CRH as PoseidonCRH, CRHScheme}, // Use Poseidon CRH
    merkle_tree::MerkleTree, // Use standard PoseidonConfig
};
use ark_ff::UniformRand;
use ark_serialize::CanonicalDeserialize;
use ark_std::vec::Vec;
use serde::{Deserialize, Serialize};
use std::{
    fs::{create_dir_all, File},
    io::{Read, Write},
    path::PathBuf,
};

// Import from our zk_circuits package
use zk_circuits::types::{ConstraintField, fr_to_hex, GachaMerkleConfig, NativePoseidonConfig}; // Use updated types

// --- Structs (Updated for clarity) ---

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ItemMaster {
    id: String, // Keep original ID for reference
    name: String,
    image_url: String,
    rarity: String,
}

// Item details for the frontend (camelCase for JavaScript)
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ItemDetails {
    id: String,
    name: String,
    #[serde(rename = "imageUrl")]
    imageUrl: String,
    rarity: String,
}

// Holds data needed for the client/prover for a specific item slot
#[derive(Serialize, Deserialize, Debug)]
struct ItemProofData {
    #[serde(rename = "itemIdHex")]
    item_id_hex: String, // Field element representation of item ID
    #[serde(rename = "secretKeyHex")]
    secret_key_hex: String, // Secret key used for this item slot
    #[serde(rename = "merklePathNodesHex")]
    merkle_path_nodes_hex: Vec<String>, // Sibling node hashes (hex)
    #[serde(rename = "leafSiblingHashHex")]
    leaf_sibling_hash_hex: String, // Leaf's sibling hash (hex)
    #[serde(rename = "leafIndex")]
    leaf_index: usize, // Index of this leaf in the tree
}

// Holds generated data before building the tree
struct GeneratedLeafData {
    item_id: ConstraintField,       // Field element ID
    secret_key: ConstraintField,    // Generated secret key
    leaf_hash: ConstraintField,     // H(secret_key, item_id)
    item_id_hex: String,  // Hex for output file
    secret_key_hex: String, // Hex for output file
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Preparing Gacha Game data (BLS12-381)...");

    // 1. Setup directories
    let output_dir = PathBuf::from("./output");
    let items_dir = output_dir.join("items"); // Directory for individual item proof files
    create_dir_all(&items_dir)?;

    // 2. Define gacha item master data
    let item_master_list = define_gacha_items();

    // 3. Load Poseidon parameters
    let poseidon_params = load_poseidon_params(&output_dir.join("params.bin"))?;

    // 4. Generate leaf data (item IDs, secret keys, leaf hashes)
    println!("Generating item leaf data...");
    // Pad the item list to the next power of 2
    let num_items = item_master_list.len();
    let tree_size = num_items.next_power_of_two();
    println!(
        "Padding item list from {} to {} for Merkle tree.",
        num_items, tree_size
    );

    let generated_leaf_data =
        generate_leaf_data(&item_master_list, tree_size, &poseidon_params)?;
    let leaf_hashes: Vec<ConstraintField> = generated_leaf_data
        .iter()
        .map(|d| d.leaf_hash)
        .collect();

    // 5. Build the Merkle tree using GachaMerkleConfig
    println!("Building Merkle tree with {} leaves...", leaf_hashes.len());
    // Provide leaf_hash_param and two_to_one_hash_param
    let merkle_tree = MerkleTree::<GachaMerkleConfig>::new_with_leaf_digest(
        &poseidon_params, // leaf_hash_param
        &poseidon_params, // two_to_one_hash_param (same params for simplicity)
        leaf_hashes,      // The pre-computed H(secret_key, item_id) values
    )?;

    // 6. Save Merkle root
    let merkle_root = merkle_tree.root();
    let merkle_root_hex = fr_to_hex(&merkle_root)?;
    save_merkle_root(&merkle_root_hex, &output_dir.join("merkle_root.hex"))?;
    println!("Merkle Root: {}", merkle_root_hex);

    // 7. Generate and save proof data for each leaf
    println!("Generating and saving Merkle proofs for all {} leaves...", tree_size);
    let mut key_list = Vec::new(); // List of file names for easy loading later

    for i in 0..tree_size {
        // Get the corresponding generated leaf data
        let leaf_data = &generated_leaf_data[i];

        // Generate Merkle proof for this leaf index
        // The leaf data provided here is [secret_key, item_id], which matches GachaMerkleConfig::Leaf
        let proof = merkle_tree.generate_proof(i)?;

        // Prepare the output structure for this item's proof file
        let item_proof_output = ItemProofData {
            item_id_hex: leaf_data.item_id_hex.clone(),
            secret_key_hex: leaf_data.secret_key_hex.clone(),
            // Convert sibling hashes in the auth path to hex
            merkle_path_nodes_hex: proof
                .auth_path
                .iter()
                .map(|h| fr_to_hex(h).expect("Hex conversion failed"))
                .collect(),
            // Convert the leaf's sibling hash to hex
            leaf_sibling_hash_hex: fr_to_hex(&proof.leaf_sibling_hash)?,
            leaf_index: proof.leaf_index, // Use index from the proof object
        };

        // Save this item's proof data to a JSON file
        let item_file_name = format!("item_{}.json", i);
        let item_path = items_dir.join(&item_file_name);
        save_item_proof_data(&item_proof_output, &item_path)?;

        key_list.push(format!("items/{}", item_file_name)); // Relative path for key list
    }

    // 8. Save the list of generated item file names
    save_key_list(&key_list, &output_dir.join("key_list.txt"))?;

    // 9. Create and save the item master data as a map using item_id_hex as keys
    let mut item_master_map = std::collections::HashMap::new();
    
    for i in 0..item_master_list.len() {
        if i < generated_leaf_data.len() {
            let item_details = ItemDetails {
                id: item_master_list[i].id.clone(),
                name: item_master_list[i].name.clone(),
                imageUrl: item_master_list[i].image_url.clone(),
                rarity: item_master_list[i].rarity.clone(),
            };
            item_master_map.insert(generated_leaf_data[i].item_id_hex.clone(), item_details);
        }
    }
    
    save_item_master_map(&item_master_map, &output_dir.join("item_master.json"))?;

    println!("Data preparation complete!");
    Ok(())
}

/// Define gacha item properties (Pokemon theme)
fn define_gacha_items() -> Vec<ItemMaster> {
    vec![
        ItemMaster { id: "pokemon_001".into(), name: "Pikachu".into(), image_url: "/gacha/images/placeholder.svg".into(), rarity: "rare".into() },
        ItemMaster { id: "pokemon_002".into(), name: "Charizard".into(), image_url: "/gacha/images/placeholder.svg".into(), rarity: "legendary".into() },
        ItemMaster { id: "pokemon_003".into(), name: "Bulbasaur".into(), image_url: "/gacha/images/placeholder.svg".into(), rarity: "uncommon".into() },
        ItemMaster { id: "pokemon_004".into(), name: "Squirtle".into(), image_url: "/gacha/images/placeholder.svg".into(), rarity: "uncommon".into() },
        ItemMaster { id: "pokemon_005".into(), name: "Jigglypuff".into(), image_url: "/gacha/images/placeholder.svg".into(), rarity: "common".into() },
        ItemMaster { id: "pokemon_006".into(), name: "Eevee".into(), image_url: "/gacha/images/placeholder.svg".into(), rarity: "uncommon".into() },
        ItemMaster { id: "pokemon_007".into(), name: "Mewtwo".into(), image_url: "/gacha/images/placeholder.svg".into(), rarity: "legendary".into() },
        ItemMaster { id: "pokemon_008".into(), name: "Snorlax".into(), image_url: "/gacha/images/placeholder.svg".into(), rarity: "rare".into() },
        ItemMaster { id: "pokemon_009".into(), name: "Gengar".into(), image_url: "/gacha/images/placeholder.svg".into(), rarity: "rare".into() },
        ItemMaster { id: "pokemon_010".into(), name: "Magikarp".into(), image_url: "/gacha/images/placeholder.svg".into(), rarity: "common".into() },
        ItemMaster { id: "pokemon_011".into(), name: "Psyduck".into(), image_url: "/gacha/images/placeholder.svg".into(), rarity: "common".into() },
        ItemMaster { id: "pokemon_012".into(), name: "Mew".into(), image_url: "/gacha/images/placeholder.svg".into(), rarity: "legendary".into() },
        // Add more items as needed... Ensure the list is padded later.
    ]
}

/// Load Poseidon parameters (same as before, ensure type matches)
fn load_poseidon_params(
    params_path: &PathBuf,
) -> Result<NativePoseidonConfig, Box<dyn std::error::Error>> {
    let mut file = File::open(params_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let params = NativePoseidonConfig::deserialize_compressed_unchecked(&buffer[..])?;
    Ok(params)
}

/// Generate leaf data: item field IDs, secret keys, and leaf hashes H(secret, item_id).
/// Pads the list to `tree_size` by repeating the last item if necessary.
fn generate_leaf_data(
    item_master_list: &[ItemMaster],
    tree_size: usize,
    poseidon_params: &NativePoseidonConfig,
) -> Result<Vec<GeneratedLeafData>, Box<dyn std::error::Error>> {
    let mut rng = ark_std::test_rng();
    let mut generated_data = Vec::with_capacity(tree_size);
    let num_master_items = item_master_list.len();

    for i in 0..tree_size {
        // Use the actual item or repeat the last item for padding
        let master_item = if i < num_master_items {
            &item_master_list[i]
        } else {
            item_master_list.last().unwrap() // Safe because tree_size >= num_master_items > 0
        };

        // Convert item ID string to a field element (simple u64 conversion for demo)
        // IMPORTANT: This is NOT cryptographically secure for real item IDs.
        // Use a collision-resistant hash (like Poseidon or SHA256) of the string ID instead.
        let item_id_num = i as u64; // Using index as ID for simplicity in this demo
                                    // let item_id_num = item_id_to_u64(&master_item.id); // Alternative
        let item_id = ConstraintField::from(item_id_num);

        // Generate a unique random secret key for this leaf slot
        let secret_key = ConstraintField::rand(&mut rng);

        // Calculate the leaf hash: Poseidon(secret_key, item_id)
        // This hash is what gets stored in the Merkle tree.
        let leaf_hash =
            PoseidonCRH::<ConstraintField>::evaluate(poseidon_params, [secret_key, item_id])?;

        generated_data.push(GeneratedLeafData {
            item_id,
            secret_key,
            leaf_hash,
            item_id_hex: fr_to_hex(&item_id)?, // Store hex for the output file
            secret_key_hex: fr_to_hex(&secret_key)?, // Store hex for the output file
        });
    }

    Ok(generated_data)
}

// Helper to convert string ID to u64 (example, potentially insecure)
fn item_id_to_u64(id: &str) -> u64 {
    id.bytes().fold(0u64, |acc, byte| acc.wrapping_shl(8) | (byte as u64))
}


// --- Save Functions (Updated for ItemProofData) ---

fn save_merkle_root(
    merkle_root_hex: &str,
    path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(path)?;
    file.write_all(merkle_root_hex.as_bytes())?;
    Ok(())
}

fn save_item_proof_data(
    item_output: &ItemProofData,
    path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(item_output)?;
    let mut file = File::create(path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

fn save_key_list(key_list: &[String], path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(path)?;
    for key in key_list {
        writeln!(file, "{}", key)?; // Use writeln! for automatic newline
    }
    Ok(())
}

fn save_item_master_data(
    item_definitions: &[ItemMaster],
    path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(item_definitions)?;
    let mut file = File::create(path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

fn save_item_master_map(
    item_map: &std::collections::HashMap<String, ItemDetails>,
    path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(item_map)?;
    let mut file = File::create(path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}
