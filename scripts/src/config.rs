// scripts/src/config.rs

use serde::{Deserialize, Serialize};

// --- Core Configuration ---
pub const TREE_HEIGHT: usize = 4; // Single source of truth for height
pub const TREE_SIZE: usize = 1 << TREE_HEIGHT; // 16

// --- Data Structures ---
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemMaster {
    pub id: String,
    pub name: String,
    pub image_url: String,
    pub rarity: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemDetails {
    pub id: String,
    pub name: String,
    #[serde(rename = "imageUrl")]
    pub imageUrl: String,
    pub rarity: String,
}

// --- Data Definition ---
pub fn define_gacha_items() -> Vec<ItemMaster> {
    // Keep your item list concise
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
    ]
}