use crate::access_control::get_onft_with_owner;
use crate::string_validation::{validate_string, StringValidationType};
use crate::ContractError;
use asset_manager::assets::AssetsManager;
use cosmwasm_std::{Addr, Api, Coin, Deps, Uint128};
use cosmwasm_std::{CosmosMsg, Storage};
use omniflix_channel_types::asset::{AssetKey, AssetSource};
use omniflix_channel_types::channel::{ChannelDetails, ChannelMetadata};
use omniflix_channel_types::msg::ReservedUsername;
use omniflix_std::types::omniflix::onft::v1beta1::{Metadata, OnftQuerier};
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
    let asset_manager = AssetsManager::new();

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
) -> Result<Vec<ReservedUsername>, ContractError> {
    reserved_usernames
        .into_iter()
        .map(|reserved_username| {
            validate_string(
                &reserved_username.username.clone(),
                StringValidationType::Username,
            )?;
            if let Some(address) = &reserved_username.address {
                api.addr_validate(&address.to_string())?;
            }
            Ok(reserved_username)
        })
        .collect()
}

pub fn validate_asset_source(
    deps: Deps,
    asset_source: AssetSource,
    owner: Addr,
    name: String,
    description: String,
    media_uri: String,
) -> Result<(), ContractError> {
    validate_string(&name, StringValidationType::AssetName)?;
    validate_string(&description, StringValidationType::Description)?;
    validate_string(&media_uri, StringValidationType::Link)?;
    match asset_source {
        AssetSource::Nft {
            collection_id,
            onft_id,
        } => {
            get_onft_with_owner(deps, collection_id, onft_id, owner.to_string())?;
            Ok(())
        }
        AssetSource::OffChain {} => Ok(()),
    }
}

pub fn generate_mint_onft_msg(
    onft_id: String,
    denom_id: String,
    contract_address: String,
    recipient: String,
    onft_data: String,
    user_name: String,
) -> (CosmosMsg, Vec<(String, String)>) {
    // Create the mint message
    let mint_onft_msg: CosmosMsg = omniflix_std::types::omniflix::onft::v1beta1::MsgMintOnft {
        id: onft_id.clone(),
        denom_id: denom_id.clone(),
        sender: contract_address,
        recipient: recipient.clone(),
        data: onft_data.clone(),
        metadata: Some(Metadata {
            media_uri: " ".to_string(),
            name: user_name.clone(),
            description: " ".to_string(),
            preview_uri: " ".to_string(),
            uri_hash: " ".to_string(),
        }),
        nsfw: false,
        extensible: false,
        royalty_share: "0".to_string(),
        transferable: true,
    }
    .into();

    // Generate detailed attributes
    let attributes = vec![
        // ONFT
        ("denom_id".to_string(), denom_id.clone()),
        ("onft_id".to_string(), onft_id.clone()),
        ("owner".to_string(), recipient.clone()),
        // Metadata
        ("name".to_string(), user_name.clone()),
        ("description".to_string(), "".to_string()),
        ("media_uri".to_string(), "".to_string()),
        ("preview_uri".to_string(), "".to_string()),
        ("uri_hash".to_string(), "".to_string()),
        // Other
        ("nsfw".to_string(), "false".to_string()),
        ("data".to_string(), onft_data.clone()),
        ("extensible".to_string(), "false".to_string()),
        ("royalty_share".to_string(), "0".to_string()),
        ("transferable".to_string(), "true".to_string()),
        ("created_at".to_string(), "".to_string()),
    ];

    (mint_onft_msg, attributes)
}
