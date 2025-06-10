use cosmwasm_std::{Binary, Env};
use rand_core::{RngCore, SeedableRng};
use rand_xoshiro::Xoshiro128PlusPlus;
use sha2::{Digest, Sha256};

/// Converts a byte to an alphanumeric character using a predefined charset
fn byte_to_alphanumeric(byte: u8) -> char {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    CHARSET[(byte % CHARSET.len() as u8) as usize] as char
}

/// Generates a random ID with a given prefix using blockchain-based entropy
///
/// This is a pseudorandom generator that uses blockchain data (block time, tx index, height)
/// combined with a provided salt to generate deterministic but unpredictable IDs.
///
/// # Arguments
/// * `salt` - Additional entropy source provided by the caller
/// * `env` - Blockchain environment containing block data
/// * `prefix` - String prefix to prepend to the generated ID
///
/// # Returns
/// A string containing the prefix followed by 32 pseudorandom alphanumeric characters
pub fn generate_random_id_with_prefix(salt: &Binary, env: &Env, prefix: &str) -> String {
    // Extract relevant data from the environment
    let tx_index: u32 = env.transaction.as_ref().map_or(0, |tx| tx.index);
    let block_time = env.block.time.nanos();
    let height = env.block.height;

    // Generate a SHA-256 hash of the salt, block time, tx_index, and height
    let hash = Sha256::digest(format!("{}{}{}{}", block_time, tx_index, height, salt).as_bytes());

    // Use the first 16 bytes from the hash as seed
    let randomness: [u8; 16] = hash[..16].try_into().unwrap();

    // Generate a random ID using the randomness
    let mut id = String::with_capacity(32);
    let mut rng = Xoshiro128PlusPlus::from_seed(randomness);
    for _ in 0..32 {
        id.push(byte_to_alphanumeric(rng.next_u32() as u8));
    }

    // Prefix the result
    format!("{}{}", prefix, &id)
}
