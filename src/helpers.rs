use cosmwasm_std::{Binary, Env, StdError};
use cosmwasm_std::{Coin, Deps, Uint128};
use omniflix_std::types::omniflix::onft::v1beta1::Onft;
use omniflix_std::types::omniflix::onft::v1beta1::OnftQuerier;
use rand_core::{RngCore, SeedableRng};
use rand_xoshiro::Xoshiro128PlusPlus;
use sha2::{Digest, Sha256};
use std::str::FromStr;

use crate::channels::ChannelDetails;

pub fn get_collection_creation_fee(deps: Deps) -> Coin {
    let onft_querier = OnftQuerier::new(&deps.querier);
    let collection_creation_fee = onft_querier
        .params()
        .unwrap()
        .params
        .unwrap()
        .denom_creation_fee
        .unwrap();
    let collection_creation_fee = Coin {
        denom: collection_creation_fee.denom,
        amount: Uint128::from_str(&collection_creation_fee.amount).unwrap(),
    };
    collection_creation_fee
}

pub fn get_onft_with_owner(
    deps: Deps,
    collection_id: String,
    onft_id: String,
    owner: String,
) -> Result<Onft, StdError> {
    // Return the ONFT if it exists and is owned by the user
    // Else, return an error
    let onft_querier = OnftQuerier::new(&deps.querier);
    let onft_response = onft_querier
        .onft(collection_id, onft_id)
        .map_err(|_| StdError::generic_err("Query ONFT failed"))?;

    let onft = onft_response
        .onft
        .ok_or_else(|| StdError::generic_err("ONFT not found"))?;

    if onft.owner != owner {
        return Err(StdError::generic_err("ONFT not owned by the user"));
    }

    Ok(onft)
}

fn byte_to_alphanumeric(byte: u8) -> char {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    CHARSET[(byte % CHARSET.len() as u8) as usize] as char
}

pub fn generate_random_id_with_prefix(salt: &Binary, env: &Env, prefix: &str) -> String {
    // Extract relevant data from the environment
    let tx_index: u32 = env.transaction.as_ref().map_or(0, |tx| tx.index);
    let block_time = env.block.time.nanos();
    let height = env.block.height;

    // Generate a SHA-256 hash of the salt, block time, tx_index, and height
    let hash = Sha256::digest(
        format!("{}{}{}{}{}", block_time, tx_index, height, prefix, salt).as_bytes(),
    );

    // Use the first 16 bytes from the hash
    let randomness: [u8; 16] = hash[..16].try_into().unwrap();

    // Generate a random ID using the randomness
    let mut id = String::with_capacity(32);
    let mut rng = Xoshiro128PlusPlus::from_seed(randomness);
    for _ in 0..32 {
        id.push(byte_to_alphanumeric(rng.next_u32() as u8));
    }
    // Prefix the result
    format!("{}{}", prefix, &id) // Ensure the string is exactly 32 characters long
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::mock_env;

    #[test]
    fn test_generate_publish_id() {
        let channel_id = generate_random_id_with_prefix(
            &Binary::from_base64("salt").unwrap(),
            &mock_env(),
            "channel",
        );

        let onft_id = generate_random_id_with_prefix(
            &Binary::from_base64("salt").unwrap(),
            &mock_env(),
            "onft",
        );
        // remove the prefixes
        let channel_id = channel_id.split_at(7).1;
        let onft_id = onft_id.split_at(4).1;

        assert_eq!(channel_id.len(), 32);
        assert_eq!(onft_id.len(), 32);
        assert_ne!(channel_id, onft_id);
    }
}
