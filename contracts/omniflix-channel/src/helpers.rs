use crate::ContractError;
use asset_manager::assets::{AssetKey, Assets};
use cosmwasm_std::{Binary, Coin, Deps, Env, Uint128};
use cosmwasm_std::{CosmosMsg, Storage};
use cw_utils::NativeBalance;
use omniflix_std::types::omniflix::onft::v1beta1::Onft;
use omniflix_std::types::omniflix::onft::v1beta1::OnftQuerier;
use rand_core::{RngCore, SeedableRng};
use rand_xoshiro::Xoshiro128PlusPlus;
use sha2::{Digest, Sha256};
use std::str::FromStr;

pub fn get_collection_creation_fee(deps: Deps) -> Result<Coin, ContractError> {
    let onft_querier = OnftQuerier::new(&deps.querier);
    let collection_creation_fee = onft_querier
        .params()
        .map_err(|_| ContractError::CollectionCreationFeeError {})?
        .params
        .ok_or_else(|| ContractError::CollectionCreationFeeError {})?
        .denom_creation_fee
        .ok_or_else(|| ContractError::CollectionCreationFeeError {})?;
    // Convert omniflix std Coin to cosmwasm Coin
    let collection_creation_fee = Coin {
        denom: collection_creation_fee.denom,
        amount: Uint128::from_str(&collection_creation_fee.amount)
            .map_err(|_| ContractError::CollectionCreationFeeError {})?,
    };
    Ok(collection_creation_fee)
}

pub fn get_onft_with_owner(
    deps: Deps,
    collection_id: String,
    onft_id: String,
    owner: String,
) -> Result<Onft, ContractError> {
    let onft_querier = OnftQuerier::new(&deps.querier);
    let onft_response = onft_querier
        .onft(collection_id.clone(), onft_id.clone())
        .map_err(|_| ContractError::OnftQueryFailed {})?;

    let onft = onft_response
        .onft
        .ok_or_else(|| ContractError::OnftNotFound {
            collection_id: collection_id.clone(),
            onft_id: onft_id.clone(),
        })?;

    if onft.owner != owner {
        return Err(ContractError::OnftNotOwned {
            collection_id: collection_id,
            onft_id: onft_id,
        });
    }

    Ok(onft)
}

pub fn check_payment(expected: Vec<Coin>, received: Vec<Coin>) -> Result<(), ContractError> {
    let mut expected_balance = NativeBalance::default();
    for coin in expected.clone() {
        expected_balance += coin;
    }

    let mut received_balance = NativeBalance::default();
    for coin in received.clone() {
        received_balance += coin;
    }

    expected_balance.normalize();
    received_balance.normalize();

    if expected_balance != received_balance {
        return Err(ContractError::PaymentError {
            expected: expected,
            received: received,
        });
    }

    Ok(())
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
    let hash = Sha256::digest(format!("{}{}{}{}", block_time, tx_index, height, salt).as_bytes());

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
/// Purpose: We can directy create a bank message but if the value is zero, that message will fail.
/// This function will check if the value is zero and if it is, it will return an empty vec. If it is not, it will return the bank message.
pub fn bank_msg_wrapper(recipient: String, amount: Vec<Coin>) -> Vec<CosmosMsg> {
    let mut final_amount = NativeBalance::default();
    for coin in amount.clone() {
        final_amount += coin;
    }
    // Remove any zero coins
    final_amount.normalize();
    // If the final amount is empty, return an empty vec
    if final_amount.is_empty() {
        return vec![];
    }
    let bank_msg: CosmosMsg = CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
        to_address: recipient,
        amount: final_amount.0,
    });
    vec![bank_msg]
}

/// Purpose: This function will filter out assets that do not exist in storage or are not visible
pub fn filter_assets_to_remove(storage: &dyn Storage, asset_keys: Vec<AssetKey>) -> Vec<AssetKey> {
    let mut asset_keys_to_remove = Vec::new();

    for asset_key in asset_keys {
        // Try loading the asset from storage
        // If the asset does not exist, add it to the list of assets to remove
        let asset_manager = Assets::new();
        let asset = asset_manager.get_asset(storage, asset_key.clone());
        if asset.is_err() {
            asset_keys_to_remove.push(asset_key);
            continue;
        }
        if asset.unwrap().is_visible == false {
            asset_keys_to_remove.push(asset_key);
            continue;
        }
    }

    asset_keys_to_remove
}

pub fn validate_username(username: &str) -> Result<(), ContractError> {
    if username.len() < 3 || username.len() > 32 {
        return Err(ContractError::InvalidUserName {});
    }
    Ok(())
}

pub fn validate_description(description: &str) -> Result<(), ContractError> {
    if description.len() < 3 || description.len() > 256 {
        return Err(ContractError::InvalidDescription {});
    }
    Ok(())
}
