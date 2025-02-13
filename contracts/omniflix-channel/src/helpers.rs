use crate::access_control::get_onft_with_owner;
use crate::string_validation::{validate_string, StringValidationType};
use crate::ContractError;
use asset_manager::assets::Assets;
use cosmwasm_std::Storage;
use cosmwasm_std::{Addr, Api, Coin, Deps, Uint128};
use omniflix_channel_types::asset::{AssetKey, AssetSource};
use omniflix_channel_types::channel::{ChannelDetails, ChannelMetadata};
use omniflix_channel_types::msg::ReservedUsername;
use omniflix_std::types::omniflix::onft::v1beta1::OnftQuerier;
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

/// Validates the channel metadata, including optional fields
pub fn validate_channel_metadata(metadata: ChannelMetadata) -> Result<(), ContractError> {
    validate_string(&metadata.channel_name, StringValidationType::ChannelName)?;

    if let Some(description) = &metadata.description {
        validate_string(description, StringValidationType::Description)?;
    }
    if let Some(profile_picture) = &metadata.profile_picture {
        validate_string(profile_picture, StringValidationType::Link)?;
    }
    if let Some(banner_picture) = &metadata.banner_picture {
        validate_string(banner_picture, StringValidationType::Link)?;
    }

    Ok(())
}

pub fn validate_channel_details(details: ChannelDetails) -> Result<(), ContractError> {
    validate_string(&details.user_name, StringValidationType::Username)?;
    Ok(())
}

/// Validates reserved usernames with their associated addresses
pub fn validate_reserved_usernames(
    reserved_usernames: Vec<ReservedUsername>,
    api: &dyn Api,
) -> Result<Vec<(String, Addr)>, ContractError> {
    reserved_usernames
        .into_iter()
        .map(|reserved_username| {
            validate_string(&reserved_username.username, StringValidationType::Username)?;
            let addr = match reserved_username.address {
                Some(address) => api.addr_validate(&address)?,
                None => Addr::unchecked(""),
            };
            Ok((reserved_username.username, addr))
        })
        .collect()
}

pub fn validate_asset_source(
    deps: Deps,
    asset_source: AssetSource,
    owner: Addr,
) -> Result<(), ContractError> {
    match asset_source {
        AssetSource::Nft {
            collection_id,
            onft_id,
        } => {
            get_onft_with_owner(deps, collection_id, onft_id, owner.to_string())?;
            Ok(())
        }
        AssetSource::OffChain {
            media_uri,
            name,
            description,
        } => {
            validate_string(&media_uri, StringValidationType::Link)?;
            validate_string(&name, StringValidationType::AssetName)?;
            validate_string(&description, StringValidationType::Description)?;
            Ok(())
        }
    }
}
