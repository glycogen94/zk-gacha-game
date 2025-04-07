use ark_std::{
    rand::{rngs::StdRng, SeedableRng},
    string::{String, ToString},
};
use getrandom::getrandom;

pub fn get_rng(seed: Option<u64>) -> Result<StdRng, String> {
    match seed {
        Some(seed) => Ok(StdRng::seed_from_u64(seed)),
        None => {
            let mut new_seed = [0u8; 32]; // u64 is 8 bytes
            if getrandom(&mut new_seed).is_err() {
                return Err("Failed to generate secure seed".to_string());
            }
            Ok(StdRng::from_seed(new_seed))
        }
    }
}
