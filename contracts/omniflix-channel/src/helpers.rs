use crate::ContractError;
use asset_manager::assets::{AssetKey, Assets};
use channel_manager::types::{ChannelDetails, ChannelMetadata};
use cosmwasm_std::{Addr, Api, Binary, Coin, Deps, Env, Uint128};
use cosmwasm_std::{CosmosMsg, Storage};
use cw_utils::NativeBalance;
use omniflix_channel_types::msg::ReservedUsername;
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

/// Purpose: Filters out assets that do not exist in storage or are not visible
pub fn filter_assets_to_remove(storage: &dyn Storage, asset_keys: Vec<AssetKey>) -> Vec<AssetKey> {
    let asset_manager = Assets::new();

    asset_keys
        .into_iter()
        .filter(
            |asset_key| match asset_manager.get_asset(storage, asset_key.clone()) {
                Ok(asset) => !asset.is_visible,
                Err(_) => true,
            },
        )
        .collect()
}

/// Validates a username to ensure it meets length and character requirements
pub fn validate_username(username: &str) -> Result<(), ContractError> {
    if !(3..=32).contains(&username.len()) {
        return Err(ContractError::InvalidUserName {});
    }
    if !username.chars().all(|c| c.is_ascii_lowercase()) {
        return Err(ContractError::InvalidUserName {});
    }
    Ok(())
}

/// Validates a description to ensure it meets length requirements
pub fn validate_description(description: &str) -> Result<(), ContractError> {
    if !(3..=256).contains(&description.len()) {
        return Err(ContractError::InvalidDescription {});
    }
    Ok(())
}

/// Validates a link to ensure it meets length requirements
pub fn validate_link(link: &str) -> Result<(), ContractError> {
    if !(3..=256).contains(&link.len()) {
        return Err(ContractError::InvalidLink {});
    }
    Ok(())
}

/// Validates a channel name to ensure it meets length and character requirements
pub fn validate_channel_name(channel_name: &str) -> Result<(), ContractError> {
    if !(3..=32).contains(&channel_name.len()) {
        return Err(ContractError::InvalidChannelName {});
    }
    if !channel_name.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(ContractError::InvalidChannelName {});
    }
    Ok(())
}

/// Validates the channel details by checking the username
pub fn validate_channel_details(details: ChannelDetails) -> Result<(), ContractError> {
    validate_username(&details.user_name)
}

/// Validates the channel metadata, including optional fields
pub fn validate_channel_metadata(metadata: ChannelMetadata) -> Result<(), ContractError> {
    validate_channel_name(&metadata.channel_name)?;

    if let Some(description) = &metadata.description {
        validate_description(description)?;
    }
    if let Some(profile_picture) = &metadata.profile_picture {
        validate_link(profile_picture)?;
    }
    if let Some(banner_picture) = &metadata.banner_picture {
        validate_link(banner_picture)?;
    }

    Ok(())
}

pub fn validate_optional_addr_list(
    addrs: Option<Vec<String>>,
    api: &dyn Api,
) -> Result<Vec<Addr>, ContractError> {
    addrs
        .unwrap_or_default()
        .into_iter()
        .map(|addr| api.addr_validate(&addr).map_err(ContractError::Std)) // Map StdError to ContractError
        .collect()
}

/// Validates reserved usernames with their associated addresses
pub fn validate_reserved_usernames(
    reserved_usernames: Vec<ReservedUsername>,
    api: &dyn Api,
) -> Result<Vec<(String, Addr)>, ContractError> {
    reserved_usernames
        .into_iter()
        .map(|reserved_username| {
            validate_username(&reserved_username.username)?;
            let addr = match reserved_username.address {
                Some(address) => api.addr_validate(&address)?,
                None => Addr::unchecked(""),
            };
            Ok((reserved_username.username, addr))
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_username() {
        // Test username too short
        let username = "ab";
        let res = validate_username(username);
        assert_eq!(res, Err(ContractError::InvalidUserName {}));

        // Test username too long
        let username = "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz";
        let res = validate_username(username);
        assert_eq!(res, Err(ContractError::InvalidUserName {}));

        // Test username with numbers
        let username = "abc123";
        let res = validate_username(username);
        assert_eq!(res, Err(ContractError::InvalidUserName {}));

        // Test username with uppercase letters
        let username = "Abcdefg";
        let res = validate_username(username);
        assert_eq!(res, Err(ContractError::InvalidUserName {}));

        // Test username with special characters
        let username = "abc-def";
        let res = validate_username(username);
        assert_eq!(res, Err(ContractError::InvalidUserName {}));

        // Test valid username with lowercase alphabet only
        let username = "channel";
        let res = validate_username(username);
        assert_eq!(res, Ok(()));

        // Test another valid username
        let username = "mintusername";
        let res = validate_username(username);
        assert_eq!(res, Ok(()));

        let username = "reserved";
        let res = validate_username(username);
        assert_eq!(res, Ok(()));
    }
}
