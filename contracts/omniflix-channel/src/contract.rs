use crate::access_control::validate_permissions;
use crate::bank_helpers::{bank_msg_wrapper, check_payment, distribute_funds_with_shares};
use crate::error::ContractError;
use crate::helpers::{
    filter_assets_to_remove, generate_mint_onft_msg, get_collection_creation_fee,
    validate_asset_source, validate_channel_details, validate_channel_metadata,
    validate_reserved_usernames,
};
use crate::random::generate_random_id_with_prefix;
use crate::state::CONFIG;
use crate::string_validation::{validate_string, StringValidationType};
use asset_manager::assets::AssetsManager;
use asset_manager::playlists::PlaylistsManager;
use channel_manager::channel::ChannelsManager;
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult,
};
use cw_utils::must_pay;
use omniflix_channel_types::asset::{Asset, AssetKey, AssetSource, Flag, Playlist};
use omniflix_channel_types::channel::{
    ChannelCollaborator, ChannelDetails, ChannelMetadata, ChannelOnftData, Role,
};
use omniflix_channel_types::config::{AuthDetails, ChannelConractConfig};
use omniflix_channel_types::msg::{
    AssetResponse, ChannelResponse, CollaboratorInfo, ExecuteMsg, InstantiateMsg, QueryMsg,
    ReservedUsername,
};
use pauser::PauseState;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Validate the admin address provided in the instantiation message
    let protocol_admin = deps
        .api
        .addr_validate(&msg.clone().protocol_admin.into_string())?;
    //Initialize the pause state and set the initial pausers
    let pause_state = PauseState::new()?;
    pause_state.set_pausers(
        deps.storage,
        info.sender.clone(),
        vec![protocol_admin.clone()],
    )?;

    // Validate the fee collector address, or default to the admin address if validation fails
    let fee_collector = deps
        .api
        .addr_validate(&msg.fee_collector.clone().into_string())?;

    // Save channel CONFIG
    let channel_contract_config = ChannelConractConfig {
        auth_details: AuthDetails {
            protocol_admin: protocol_admin.clone(),
            fee_collector: fee_collector.clone(),
        },
        accepted_tip_denoms: msg.accepted_tip_denoms.clone(),
        channels_collection_id: msg.channels_collection_id.clone(),
        channels_collection_name: msg.channels_collection_name.clone(),
        channels_collection_symbol: msg.channels_collection_symbol.clone(),
        channel_creation_fee: msg.channel_creation_fee.clone(),
    };
    // Save the channel CONFIG to the contract state
    CONFIG.save(deps.storage, &channel_contract_config)?;

    // Query the collection creation fee from onft module
    let collection_creation_fee = get_collection_creation_fee(deps.as_ref())?;

    // Check if the payment provided in the message matches the required creation fee
    check_payment(
        [collection_creation_fee.clone()].to_vec(),
        info.funds.clone(),
    )?;

    let channels_manager = ChannelsManager::new();
    validate_reserved_usernames(msg.reserved_usernames.clone(), deps.api)?;
    channels_manager.add_reserved_usernames(deps.storage, msg.reserved_usernames.clone())?;

    // Prepare the message to create a new ONFT denom (collection)
    let onft_creation_message: CosmosMsg =
        omniflix_std::types::omniflix::onft::v1beta1::MsgCreateDenom {
            id: msg.channels_collection_id.clone(),
            name: msg.channels_collection_name.clone(),
            symbol: msg.channels_collection_symbol.clone(),
            creation_fee: Some(collection_creation_fee.into()),
            sender: env.contract.address.clone().to_string(),
            ..Default::default()
        }
        .into();

    let response = Response::new()
        .add_message(onft_creation_message)
        .add_attribute("action", "instantiate")
        .add_attribute("channels_collection_id", msg.channels_collection_id.clone())
        .add_attribute(
            "channels_collection_name",
            msg.channels_collection_name.clone(),
        )
        .add_attribute(
            "channels_collection_symbol",
            msg.channels_collection_symbol.clone(),
        );

    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AdminRemoveAssets {
            asset_keys,
            refresh_flags,
        } => remove_assets(deps, info, asset_keys, refresh_flags),
        ExecuteMsg::AssetFlag {
            channel_id,
            publish_id,
            flag,
        } => flag_asset(deps, info, channel_id, publish_id, flag),
        ExecuteMsg::Pause {} => pause(deps, info),
        ExecuteMsg::Unpause {} => unpause(deps, info),
        ExecuteMsg::SetPausers { pausers } => set_pausers(deps, info, pausers),
        ExecuteMsg::AssetPublish {
            asset_source,
            salt,
            channel_id,
            playlist_name,
            is_visible,
            name,
            description,
            media_uri,
        } => publish(
            deps,
            env,
            info,
            asset_source,
            salt,
            channel_id,
            playlist_name,
            is_visible,
            name,
            description,
            media_uri,
        ),
        ExecuteMsg::AssetUnpublish {
            publish_id,
            channel_id,
        } => unpublish(deps, info, publish_id, channel_id),
        ExecuteMsg::PlaylistRefresh {
            channel_id,
            playlist_name,
        } => refresh_playlist(deps, info, channel_id, playlist_name),
        ExecuteMsg::PlaylistCreate {
            playlist_name,
            channel_id,
        } => create_playlist(deps, info, channel_id, playlist_name),
        ExecuteMsg::ChannelCreate {
            user_name,
            channel_name,
            salt,
            description,
            profile_picture,
            banner_picture,
            payment_address,
        } => create_channel(
            deps,
            env,
            info,
            salt,
            description,
            payment_address,
            user_name,
            channel_name,
            profile_picture,
            banner_picture,
        ),
        ExecuteMsg::ChannelUpdateDetails {
            channel_id,
            description,
            channel_name,
            profile_picture,
            banner_picture,
            payment_address,
        } => update_channel_details(
            deps,
            info,
            channel_id,
            description,
            channel_name,
            profile_picture,
            banner_picture,
            payment_address,
        ),

        ExecuteMsg::PlaylistDelete {
            playlist_name,
            channel_id,
        } => delete_playlist(deps, info, channel_id, playlist_name),
        ExecuteMsg::PlaylistRemoveAsset {
            publish_id,
            channel_id,
            playlist_name,
        } => remove_asset_from_playlist(deps, info, publish_id, channel_id, playlist_name),
        ExecuteMsg::AdminSetConfig {
            channel_creation_fee,
            protocol_admin,
            fee_collector,
        } => set_config(
            deps,
            info,
            channel_creation_fee,
            protocol_admin,
            fee_collector,
        ),
        ExecuteMsg::PlaylistAddAsset {
            publish_id,
            asset_channel_id,
            channel_id,
            playlist_name,
        } => add_asset_to_playlist(
            deps,
            info,
            asset_channel_id,
            publish_id,
            channel_id,
            playlist_name,
        ),
        ExecuteMsg::AssetUpdateDetails {
            publish_id,
            channel_id,
            is_visible,
            name,
            description,
            media_uri,
        } => update_asset_details(
            deps,
            info,
            publish_id,
            channel_id,
            is_visible,
            name,
            description,
            media_uri,
        ),
        ExecuteMsg::ChannelDelete { channel_id } => delete_channel(deps, info, channel_id),
        ExecuteMsg::AdminManageReservedUsernames {
            add_usernames,
            remove_usernames,
        } => manage_reserved_usernames(deps, info, add_usernames, remove_usernames),
        ExecuteMsg::ChannelTip { channel_id, amount } => {
            tip_channel(deps, info, channel_id, amount)
        }
        ExecuteMsg::ChannelAddCollaborator {
            channel_id,
            collaborator_address,
            collaborator_details,
        } => add_collaborator(
            deps,
            info,
            channel_id,
            collaborator_address,
            collaborator_details,
        ),
        ExecuteMsg::ChannelRemoveCollaborator {
            channel_id,
            collaborator_address,
        } => remove_collaborator(deps, info, channel_id, collaborator_address),
        ExecuteMsg::ChannelFollow { channel_id } => follow_channel(deps, info, channel_id),
        ExecuteMsg::ChannelUnfollow { channel_id } => unfollow_channel(deps, info, channel_id),
    }
}

fn create_channel(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    salt: Binary,
    description: Option<String>,
    payment_address: Addr,
    user_name: String,
    channel_name: String,
    profile_picture: Option<String>,
    banner_picture: Option<String>,
) -> Result<Response, ContractError> {
    let pause_state = PauseState::new()?;
    pause_state.error_if_paused(deps.storage)?;

    let config = CONFIG.load(deps.storage)?;

    // Check if the payment provided in the message matches the required creation fee
    check_payment(config.channel_creation_fee.clone(), info.funds.clone())?;

    // Generate a random channel onft ID
    let onft_id = generate_random_id_with_prefix(&salt, &env, "onft");

    // Generate a random channel ID
    let channel_id = generate_random_id_with_prefix(&salt, &env, "channel");

    let channels_manager = ChannelsManager::new();

    let channel_details = ChannelDetails {
        channel_id: channel_id.clone(),
        user_name: user_name.clone(),
        onft_id: onft_id.clone(),
        payment_address: deps.api.addr_validate(&payment_address.into_string())?,
    };
    let channel_metadata = ChannelMetadata {
        channel_name: channel_name.clone(),
        description: description.clone(),
        profile_picture: profile_picture.clone(),
        banner_picture: banner_picture.clone(),
    };
    validate_channel_details(channel_details.clone())?;
    validate_channel_metadata(channel_metadata.clone())?;

    // Check if the username is reserved
    match channels_manager.get_reserved_status(deps.storage, user_name.clone())? {
        None => {
            // Username is not reserved, proceed with channel creation
        }
        Some(reserved_addr) => {
            if reserved_addr != Some(info.sender.clone()) {
                // Username is reserved but not assigned to anyone
                return Err(ContractError::UserNameReserved {});
            }
            // Username is reserved for a specific address
            if reserved_addr != Some(info.sender.clone()) {
                return Err(ContractError::UserNameReserved {});
            }
            // Sender matches the reserved address, remove the reservation
            channels_manager.remove_reserved_usernames(deps.storage, vec![user_name.clone()])?;
        }
    }
    // Add the new channel to the collection
    // Checks for uniqueness of the channel ID and username
    channels_manager.add_channel(
        deps.storage,
        channel_id.clone(),
        channel_details.clone(),
        channel_metadata.clone(),
    )?;

    // Create the onft data for the channel. This data will be stored in the onft's data field
    let onft_data = ChannelOnftData {
        channel_id: channel_id.clone(),
        user_name: user_name.clone(),
        onft_id: onft_id.clone(),
    };

    let string_onft_data =
        serde_json::to_string(&onft_data).map_err(|_| ContractError::InvalidOnftData {})?;

    // Generate the mint message and its attributes
    let (mint_onft_msg, nft_attributes) = generate_mint_onft_msg(
        onft_id.clone(),
        config.channels_collection_id.clone(),
        env.contract.address.clone().to_string(),
        info.sender.clone().to_string(),
        string_onft_data,
        user_name.clone(),
    );

    // Pay the channel creation fee to the fee collector
    let bank_channel_fee_msg = bank_msg_wrapper(
        config.auth_details.fee_collector,
        config.channel_creation_fee,
    );

    let response = Response::new()
        .add_message(mint_onft_msg)
        .add_messages(bank_channel_fee_msg)
        .add_attribute("action", "register_channel")
        .add_attributes(nft_attributes);
    Ok(response)
}

fn follow_channel(
    deps: DepsMut,
    info: MessageInfo,
    channel_id: String,
) -> Result<Response, ContractError> {
    let channels_manager = ChannelsManager::new();
    channels_manager.add_follower(deps.storage, channel_id.clone(), info.sender.clone())?;

    let response = Response::new()
        .add_attribute("action", "follow_channel")
        .add_attribute("channel_id", channel_id);
    Ok(response)
}

fn unfollow_channel(
    deps: DepsMut,
    info: MessageInfo,
    channel_id: String,
) -> Result<Response, ContractError> {
    let channels_manager = ChannelsManager::new();
    channels_manager.remove_follower(deps.storage, channel_id.clone(), info.sender.clone())?;

    let response = Response::new()
        .add_attribute("action", "unfollow_channel")
        .add_attribute("channel_id", channel_id);
    Ok(response)
}

fn delete_channel(
    deps: DepsMut,
    info: MessageInfo,
    channel_id: String,
) -> Result<Response, ContractError> {
    let pause_state = PauseState::new()?;
    pause_state.error_if_paused(deps.storage)?;

    let config = CONFIG.load(deps.storage)?;

    let channels_manager = ChannelsManager::new();
    let assets_manager = AssetsManager::new();
    let playlist_manager = PlaylistsManager::new();
    // Check if the sender has admin permissions
    validate_permissions(
        deps.as_ref(),
        channel_id.clone(),
        info.sender.clone(),
        config.channels_collection_id.clone(),
        Role::Admin,
    )?;

    channels_manager.delete_channel(deps.storage, channel_id.clone())?;
    assets_manager.delete_assets_by_channel_id(deps.storage, channel_id.clone())?;
    playlist_manager.delete_playlists_by_channel_id(deps.storage, channel_id.clone());

    let response = Response::new()
        .add_attribute("action", "delete_channel")
        .add_attribute("channel_id", channel_id);

    Ok(response)
}

fn publish(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    asset_source: AssetSource,
    salt: Binary,
    channel_id: String,
    playlist_name: Option<String>,
    is_visible: bool,
    name: String,
    description: String,
    media_uri: String,
) -> Result<Response, ContractError> {
    let pause_state = PauseState::new()?;
    pause_state.error_if_paused(deps.storage)?;

    let config = CONFIG.load(deps.storage)?;

    validate_permissions(
        deps.as_ref(),
        channel_id.clone(),
        info.sender.clone(),
        config.channels_collection_id.clone(),
        Role::Publisher,
    )?;

    let publish_id = generate_random_id_with_prefix(&salt, &env, "publish");

    validate_asset_source(
        deps.as_ref(),
        asset_source.clone(),
        info.sender.clone(),
        name.clone(),
        description.clone(),
        media_uri.clone(),
    )?;

    // Define the asset to be published
    let asset = Asset {
        channel_id: channel_id.clone(),
        publish_id: publish_id.clone(),
        asset_source: asset_source.clone(),
        is_visible: is_visible,
        name: name.clone(),
        description: description.clone(),
        media_uri: media_uri.clone(),
    };

    // Add asset to the channel's asset list
    let assets_manager = AssetsManager::new();
    let asset_key = (channel_id.clone(), publish_id.clone());
    assets_manager.add_asset(deps.storage, asset_key.clone(), asset.clone())?;

    if let Some(playlist_name) = playlist_name.clone() {
        if is_visible {
            let playlists_manager = PlaylistsManager::new();
            playlists_manager.add_asset_to_playlist(
                deps.storage,
                channel_id.clone(),
                playlist_name.clone(),
                asset_key,
            )?;
        }
    }

    let mut response = Response::new()
        .add_attribute("action", "publish")
        .add_attribute("publish_id", publish_id)
        .add_attribute("channel_id", channel_id)
        .add_attribute("asset_source", asset_source.to_string());

    if let Some(playlist_name) = playlist_name {
        response = response.add_attribute("playlist_name", playlist_name);
    }
    Ok(response)
}

fn unpublish(
    deps: DepsMut,
    info: MessageInfo,
    publish_id: String,
    channel_id: String,
) -> Result<Response, ContractError> {
    let pause_state = PauseState::new()?;
    pause_state.error_if_paused(deps.storage)?;

    let config = CONFIG.load(deps.storage)?;

    validate_permissions(
        deps.as_ref(),
        channel_id.clone(),
        info.sender.clone(),
        config.channels_collection_id.clone(),
        Role::Publisher,
    )?;

    let assets_manager = AssetsManager::new();
    let asset_key = (channel_id.clone(), publish_id.clone());
    assets_manager.delete_assets(deps.storage, vec![asset_key.clone()])?;

    let response = Response::new()
        .add_attribute("action", "unpublish")
        .add_attribute("publish_id", publish_id)
        .add_attribute("channel_id", channel_id);

    Ok(response)
}

fn refresh_playlist(
    deps: DepsMut,
    info: MessageInfo,
    channel_id: String,
    playlist_name: String,
) -> Result<Response, ContractError> {
    let pause_state = PauseState::new()?;
    pause_state.error_if_paused(deps.storage)?;

    let config = CONFIG.load(deps.storage)?;

    validate_permissions(
        deps.as_ref(),
        channel_id.clone(),
        info.sender.clone(),
        config.channels_collection_id.clone(),
        Role::Publisher,
    )?;

    let playlists_manager = PlaylistsManager::new();
    let playlist_asset_keys = playlists_manager
        .get_playlist(deps.storage, channel_id.clone(), playlist_name.clone())?
        .assets;
    let asset_keys_to_remove = filter_assets_to_remove(deps.storage, playlist_asset_keys.clone());

    playlists_manager.remove_assets_from_playlist(
        deps.storage,
        channel_id.clone(),
        playlist_name.clone(),
        asset_keys_to_remove.clone(),
    )?;
    let removed_publish_ids: Vec<String> = asset_keys_to_remove
        .iter()
        .map(|asset_key| asset_key.1.clone())
        .collect();

    let response = Response::new()
        .add_attribute("action", "refresh_playlist")
        .add_attribute("channel_id", channel_id)
        .add_attribute("playlist_name", playlist_name)
        .add_attribute("removed_publish_ids", removed_publish_ids.join(", "));

    Ok(response)
}

fn create_playlist(
    deps: DepsMut,
    info: MessageInfo,
    channel_id: String,
    playlist_name: String,
) -> Result<Response, ContractError> {
    let pause_state = PauseState::new()?;
    pause_state.error_if_paused(deps.storage)?;

    let config = CONFIG.load(deps.storage)?;

    validate_permissions(
        deps.as_ref(),
        channel_id.clone(),
        info.sender.clone(),
        config.channels_collection_id.clone(),
        Role::Publisher,
    )?;

    let playlists_manager = PlaylistsManager::new();
    playlists_manager.add_new_playlist(deps.storage, channel_id.clone(), playlist_name.clone())?;

    let response = Response::new()
        .add_attribute("action", "create_playlist")
        .add_attribute("channel_id", channel_id)
        .add_attribute("playlist_name", playlist_name);

    Ok(response)
}

fn pause(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let pause_state = PauseState::new()?;
    pause_state.pause(deps.storage, &info.sender)?;

    let response = Response::new()
        .add_attribute("action", "pause")
        .add_attribute("pauser", info.sender.clone().to_string());
    Ok(response)
}

fn unpause(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let pause_state = PauseState::new()?;
    pause_state.unpause(deps.storage, &info.sender)?;

    let response = Response::new()
        .add_attribute("action", "unpause")
        .add_attribute("pauser", info.sender.clone().to_string());
    Ok(response)
}

fn set_pausers(
    deps: DepsMut,
    info: MessageInfo,
    pausers: Vec<String>,
) -> Result<Response, ContractError> {
    let pause_state = PauseState::new()?;
    // Validate pauser addresses
    let validated_pausers: Vec<Addr> = pausers
        .iter()
        .map(|pauser| deps.api.addr_validate(pauser))
        .collect::<Result<Vec<Addr>, _>>()?;

    for pauser in pausers.clone() {
        deps.api.addr_validate(&pauser)?;
    }
    pause_state.set_pausers(deps.storage, info.sender.clone(), validated_pausers)?;

    let response = Response::new()
        .add_attribute("action", "set_pausers")
        .add_attribute("pauser", info.sender.clone().to_string());
    Ok(response)
}

fn add_collaborator(
    deps: DepsMut,
    info: MessageInfo,
    channel_id: String,
    collaborator_address: String,
    collaborator_details: ChannelCollaborator,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    validate_permissions(
        deps.as_ref(),
        channel_id.clone(),
        info.sender.clone(),
        config.channels_collection_id.clone(),
        Role::Admin,
    )?;

    // Validate the collaborator address
    let collaborator_address = deps.api.addr_validate(&collaborator_address)?;
    let channels_manager = ChannelsManager::new();

    // Add the collaborator to the channel
    channels_manager.add_collaborator(
        deps.storage,
        channel_id.clone(),
        collaborator_address.clone(),
        collaborator_details.clone(),
    )?;

    let response = Response::new()
        .add_attribute("action", "add_collaborator")
        .add_attribute("channel_id", channel_id)
        .add_attribute("collaborator_address", collaborator_address)
        .add_attribute("share", collaborator_details.share.to_string());

    Ok(response)
}

fn remove_collaborator(
    deps: DepsMut,
    info: MessageInfo,
    channel_id: String,
    collaborator_address: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    validate_permissions(
        deps.as_ref(),
        channel_id.clone(),
        info.sender.clone(),
        config.channels_collection_id.clone(),
        Role::Admin,
    )?;

    // Validate the collaborator address
    let collaborator_address = deps.api.addr_validate(&collaborator_address)?;

    // Remove the collaborator from the channel
    let channels_manager = ChannelsManager::new();
    channels_manager.remove_collaborator(
        deps.storage,
        channel_id.clone(),
        collaborator_address.clone(),
    )?;

    let response = Response::new()
        .add_attribute("action", "remove_collaborator")
        .add_attribute("channel_id", channel_id)
        .add_attribute("collaborator_address", collaborator_address);

    Ok(response)
}

fn update_channel_details(
    deps: DepsMut,
    info: MessageInfo,
    channel_id: String,
    description: Option<String>,
    channel_name: Option<String>,
    profile_picture: Option<String>,
    banner_picture: Option<String>,
    payment_address: Option<String>,
) -> Result<Response, ContractError> {
    let pause_state = PauseState::new()?;
    pause_state.error_if_paused(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    let channels_collection_id = config.channels_collection_id.clone();

    let channel_manager = ChannelsManager::new();

    validate_permissions(
        deps.as_ref(),
        channel_id.clone(),
        info.sender.clone(),
        channels_collection_id.clone(),
        Role::Admin,
    )?;

    let mut channel_metadata =
        channel_manager.get_channel_metadata(deps.storage, channel_id.clone())?;

    if let Some(description) = description.clone() {
        channel_metadata.description = Some(description.clone());
    }

    if let Some(channel_name) = channel_name.clone() {
        channel_metadata.channel_name = channel_name.clone();
    }

    if let Some(profile_picture) = profile_picture.clone() {
        channel_metadata.profile_picture = Some(profile_picture.clone());
    }

    if let Some(banner_picture) = banner_picture.clone() {
        channel_metadata.banner_picture = Some(banner_picture.clone());
    }
    validate_channel_metadata(channel_metadata.clone())?;

    channel_manager.update_channel_metadata(
        deps.storage,
        channel_id.clone(),
        channel_metadata.clone(),
    )?;

    if let Some(payment_address) = payment_address.clone() {
        let payment_address = deps.api.addr_validate(&payment_address)?;
        channel_manager.update_payment_address(
            deps.storage,
            channel_id.clone(),
            payment_address,
        )?;
    }

    let response = Response::new()
        .add_attribute("action", "update_channel_details")
        .add_attribute("channel_id", channel_id);

    Ok(response)
}

fn delete_playlist(
    deps: DepsMut,
    info: MessageInfo,
    channel_id: String,
    playlist_name: String,
) -> Result<Response, ContractError> {
    let pause_state = PauseState::new()?;
    pause_state.error_if_paused(deps.storage)?;

    let config = CONFIG.load(deps.storage)?;

    validate_permissions(
        deps.as_ref(),
        channel_id.clone(),
        info.sender.clone(),
        config.channels_collection_id.clone(),
        Role::Publisher,
    )?;

    let playlist_manager = PlaylistsManager::new();
    playlist_manager.delete_playlist(deps.storage, channel_id.clone(), playlist_name.clone())?;

    let response = Response::new()
        .add_attribute("action", "delete_playlist")
        .add_attribute("channel_id", channel_id)
        .add_attribute("playlist_name", playlist_name);

    Ok(response)
}
fn add_asset_to_playlist(
    deps: DepsMut,
    info: MessageInfo,
    asset_channel_id: String,
    publish_id: String,
    channel_id: String,
    playlist_name: String,
) -> Result<Response, ContractError> {
    let pause_state = PauseState::new()?;
    pause_state.error_if_paused(deps.storage)?;

    let config = CONFIG.load(deps.storage)?;

    validate_permissions(
        deps.as_ref(),
        channel_id.clone(),
        info.sender.clone(),
        config.channels_collection_id.clone(),
        Role::Publisher,
    )?;

    let playlist_manager = PlaylistsManager::new();

    // Load the asset
    let assets_manager = AssetsManager::new();
    let asset_key = (asset_channel_id.clone(), publish_id.clone());
    let asset = assets_manager.get_asset(deps.storage, asset_key.clone())?;

    // Verify that the asset is visible
    if asset.is_visible == false {
        return Err(ContractError::AssetNotVisible {});
    }

    // Add asset to playlist
    playlist_manager.add_asset_to_playlist(
        deps.storage,
        channel_id.clone(),
        playlist_name.clone(),
        asset_key.clone(),
    )?;

    let response = Response::new()
        .add_attribute("action", "add_asset_to_playlist")
        .add_attribute("channel_id", channel_id)
        .add_attribute("playlist_name", playlist_name)
        .add_attribute("publish_id", publish_id);

    Ok(response)
}
fn remove_asset_from_playlist(
    deps: DepsMut,
    info: MessageInfo,
    publish_id: String,
    channel_id: String,
    playlist_name: String,
) -> Result<Response, ContractError> {
    let pause_state = PauseState::new()?;
    pause_state.error_if_paused(deps.storage)?;

    let config = CONFIG.load(deps.storage)?;

    validate_permissions(
        deps.as_ref(),
        channel_id.clone(),
        info.sender.clone(),
        config.channels_collection_id.clone(),
        Role::Publisher,
    )?;

    let playlists_manager = PlaylistsManager::new();
    let asset_key = (channel_id.clone(), publish_id.clone());
    // Remove the asset from the playlist
    playlists_manager.remove_assets_from_playlist(
        deps.storage,
        channel_id.clone(),
        playlist_name.clone(),
        [asset_key.clone()].to_vec(),
    )?;

    let response = Response::new()
        .add_attribute("action", "remove_asset_from_playlist")
        .add_attribute("channel_id", channel_id)
        .add_attribute("playlist_name", playlist_name)
        .add_attribute("publish_id", publish_id);

    Ok(response)
}

fn update_asset_details(
    deps: DepsMut,
    info: MessageInfo,
    publish_id: String,
    channel_id: String,
    is_visible: Option<bool>,
    name: Option<String>,
    description: Option<String>,
    media_uri: Option<String>,
) -> Result<Response, ContractError> {
    let pause_state = PauseState::new()?;
    pause_state.error_if_paused(deps.storage)?;

    let config = CONFIG.load(deps.storage)?;

    validate_permissions(
        deps.as_ref(),
        channel_id.clone(),
        info.sender.clone(),
        config.channels_collection_id.clone(),
        Role::Publisher,
    )?;

    let assets_manager = AssetsManager::new();
    let asset_key = (channel_id.clone(), publish_id.clone());
    let mut asset = assets_manager.get_asset(deps.storage, asset_key.clone())?;

    // Validate the asset name
    if let Some(name) = name {
        validate_string(&name, StringValidationType::AssetName)?;
        asset.name = name;
    }
    // Validate the asset description
    if let Some(description) = description {
        validate_string(&description, StringValidationType::Description)?;
        asset.description = description;
    }
    // Validate the asset media URI
    if let Some(media_uri) = media_uri {
        validate_string(&media_uri, StringValidationType::Link)?;
        asset.media_uri = media_uri;
    }
    if let Some(is_visible) = is_visible {
        asset.is_visible = is_visible;
    }

    assets_manager.update_asset(deps.storage, asset_key.clone(), asset.clone())?;

    let response = Response::new()
        .add_attribute("action", "update_asset_details")
        .add_attribute("channel_id", channel_id)
        .add_attribute("publish_id", publish_id)
        .add_attribute("is_visible", asset.is_visible.to_string())
        .add_attribute("name", asset.name.clone())
        .add_attribute("description", asset.description.clone())
        .add_attribute("media_uri", asset.media_uri.clone());

    Ok(response)
}

fn set_config(
    deps: DepsMut,
    info: MessageInfo,
    channel_creation_fee: Option<Vec<Coin>>,
    protocol_admin: Option<String>,
    fee_collector: Option<String>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if info.sender != config.auth_details.protocol_admin {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(protocol_admin) = protocol_admin {
        let protocol_admin = deps.api.addr_validate(&protocol_admin)?;
        config.auth_details.protocol_admin = protocol_admin;
    }

    if let Some(fee_collector) = fee_collector {
        let fee_collector = deps.api.addr_validate(&fee_collector)?;
        config.auth_details.fee_collector = fee_collector;
    }

    if let Some(channel_creation_fee) = channel_creation_fee {
        config.channel_creation_fee = channel_creation_fee;
    }

    CONFIG.save(deps.storage, &config)?;

    let response = Response::new()
        .add_attribute("action", "set_config")
        .add_attribute(
            "protocol_admin",
            config.auth_details.protocol_admin.to_string(),
        )
        .add_attribute(
            "fee_collector",
            config.auth_details.fee_collector.to_string(),
        );

    Ok(response)
}

fn manage_reserved_usernames(
    deps: DepsMut,
    info: MessageInfo,
    add_usernames: Option<Vec<ReservedUsername>>,
    remove_usernames: Option<Vec<String>>,
) -> Result<Response, ContractError> {
    let pause_state = PauseState::new()?;
    pause_state.error_if_paused(deps.storage)?;

    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.auth_details.protocol_admin {
        return Err(ContractError::Unauthorized {});
    }

    let channels_manager = ChannelsManager::new();

    let mut attrs = vec![(
        "action".to_string(),
        "manage_reserved_usernames".to_string(),
    )];
    if let Some(add_usernames) = add_usernames {
        validate_reserved_usernames(add_usernames.clone(), deps.api)?;
        channels_manager.add_reserved_usernames(deps.storage, add_usernames.clone())?;
        for username in add_usernames {
            attrs.push(("add_username".to_string(), username.username)); // Directly push owned username
        }
    }
    if let Some(remove_usernames) = remove_usernames {
        for username in remove_usernames {
            channels_manager.remove_reserved_usernames(deps.storage, vec![username.clone()])?;
            attrs.push(("remove_username".to_string(), username)); // Directly push owned username
        }
    }

    let response = Response::new().add_attributes(attrs);

    Ok(response)
}

fn tip_channel(
    deps: DepsMut,
    info: MessageInfo,
    channel_id: String,
    amount: Coin,
) -> Result<Response, ContractError> {
    let pause_state = PauseState::new()?;
    pause_state.error_if_paused(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    let accepted_tip_denoms = config.accepted_tip_denoms.clone();
    if !accepted_tip_denoms.contains(&amount.denom) {
        return Err(ContractError::InvalidTipDenom {});
    }

    let sent_amount = must_pay(&info, &amount.denom)?;

    if sent_amount != amount.amount {
        return Err(ContractError::InvalidTipAmount {});
    }

    let channels_manager = ChannelsManager::new();
    let channel_details = channels_manager.get_channel_details(deps.storage, channel_id.clone())?;
    let channel_payment_address = channel_details.payment_address.clone();
    // Calculates the shares of the collaborators
    let collaborator_shares =
        channels_manager.get_collaborator_shares(deps.storage, channel_id.clone())?;
    // Distributes the funds to the collaborators and remaining to the channel payment address
    let (bank_msgs, attributes) =
        distribute_funds_with_shares(collaborator_shares, amount.clone(), channel_payment_address)?;

    let response = Response::new()
        .add_messages(bank_msgs)
        .add_attributes(attributes)
        .add_attribute("action", "tip_creator")
        .add_attribute("channel_id", channel_id)
        .add_attribute("amount", amount.to_string());

    Ok(response)
}

fn remove_assets(
    deps: DepsMut,
    info: MessageInfo,
    asset_keys: Vec<AssetKey>,
    refresh_flags: Option<bool>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.auth_details.protocol_admin {
        return Err(ContractError::Unauthorized {});
    }

    let mut deleted_assets: Vec<AssetKey> = Vec::new();
    deleted_assets.extend(asset_keys.clone());

    // First remove the assets specified in the message
    let assets_manager = AssetsManager::new();
    assets_manager.delete_assets(deps.storage, asset_keys)?;

    // Refresh the flags if set
    // If the flag is set to true, all flags of all assets will be removed
    if let Some(refresh_flags) = refresh_flags {
        if refresh_flags {
            assets_manager.remove_all_flags(deps.storage)?;
        }
    }

    Ok(Response::new()
        .add_attribute("action", "remove_assets")
        .add_attribute("admin", config.auth_details.protocol_admin.to_string()))
}

fn flag_asset(
    deps: DepsMut,
    _info: MessageInfo,
    channel_id: String,
    publish_id: String,
    flag: Flag,
) -> Result<Response, ContractError> {
    let pause_state = PauseState::new()?;
    pause_state.error_if_paused(deps.storage)?;

    let assets_manager = AssetsManager::new();
    assets_manager.add_flag(
        deps.storage,
        channel_id.clone(),
        publish_id.clone(),
        flag.clone(),
    )?;

    Ok(Response::new()
        .add_attribute("action", "add_flag")
        .add_attribute("channel_id", channel_id)
        .add_attribute("publish_id", publish_id)
        .add_attribute("flag", flag.to_string()))
}
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Channel { channel_id } => to_json_binary(&query_channel(deps, channel_id)?),
        QueryMsg::IsPaused {} => to_json_binary(&query_is_paused(deps)?),
        QueryMsg::Pausers {} => to_json_binary(&query_pausers(deps)?),
        QueryMsg::ChannelDetails { channel_id } => {
            to_json_binary(&query_channel_details(deps, channel_id)?)
        }
        QueryMsg::ChannelMetadata { channel_id } => {
            to_json_binary(&query_channel_metadata(deps, channel_id)?)
        }
        QueryMsg::Playlist {
            channel_id,
            playlist_name,
        } => to_json_binary(&query_playlist(deps, channel_id, playlist_name)?),
        QueryMsg::Channels { start_after, limit } => {
            to_json_binary(&query_channels(deps, start_after, limit)?)
        }
        QueryMsg::ChannelId { user_name } => to_json_binary(&query_channel_id(deps, user_name)?),
        QueryMsg::Playlists {
            channel_id,
            start_after,
            limit,
        } => to_json_binary(&query_playlists(deps, channel_id, start_after, limit)?),
        QueryMsg::Config {} => to_json_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::Assets {
            channel_id,
            start_after,
            limit,
        } => to_json_binary(&query_assets(deps, channel_id, start_after, limit)?),
        QueryMsg::Asset {
            channel_id,
            publish_id,
        } => to_json_binary(&query_asset(deps, channel_id, publish_id)?),
        QueryMsg::ReservedUsernames { start_after, limit } => {
            to_json_binary(&query_reserved_usernames(deps, start_after, limit)?)
        }
        QueryMsg::GetChannelCollaborator {
            channel_id,
            collaborator_address,
        } => to_json_binary(&query_channel_collaborator(
            deps,
            channel_id,
            collaborator_address,
        )?),
        QueryMsg::GetChannelCollaborators {
            channel_id,
            start_after,
            limit,
        } => to_json_binary(&query_channel_collaborators(
            deps,
            channel_id,
            start_after,
            limit,
        )?),
        QueryMsg::FollowersCount { channel_id } => {
            to_json_binary(&query_followers_count(deps, channel_id)?)
        }
        QueryMsg::Followers {
            channel_id,
            start_after,
            limit,
        } => to_json_binary(&query_followers(deps, channel_id, start_after, limit)?),
    }
}

fn query_channel_details(deps: Deps, channel_id: String) -> Result<ChannelDetails, ContractError> {
    let channels_manager = ChannelsManager::new();
    let channel_details = channels_manager.get_channel_details(deps.storage, channel_id.clone())?;
    Ok(channel_details)
}

fn query_channel_metadata(
    deps: Deps,
    channel_id: String,
) -> Result<ChannelMetadata, ContractError> {
    let channels_manager = ChannelsManager::new();
    let channel_metadata =
        channels_manager.get_channel_metadata(deps.storage, channel_id.clone())?;
    Ok(channel_metadata)
}

fn query_channel(deps: Deps, channel_id: String) -> Result<ChannelResponse, ContractError> {
    let channels_manager = ChannelsManager::new();
    let channel_details = channels_manager.get_channel_details(deps.storage, channel_id.clone())?;
    let channel_metadata =
        channels_manager.get_channel_metadata(deps.storage, channel_id.clone())?;
    let channel_collaborators =
        channels_manager.get_channel_collaborators(deps.storage, channel_id.clone(), None, None)?;
    let follower_count = channels_manager.get_followers_count(deps.storage, channel_id.clone())?;

    Ok(ChannelResponse {
        channel_id: channel_details.channel_id,
        user_name: channel_details.user_name,
        onft_id: channel_details.onft_id,
        payment_address: channel_details.payment_address.to_string(),
        channel_name: channel_metadata.channel_name,
        description: channel_metadata.description,
        profile_picture: channel_metadata.profile_picture,
        banner_picture: channel_metadata.banner_picture,
        collaborators: channel_collaborators,
        follower_count,
    })
}

fn query_channels(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<ChannelResponse>, ContractError> {
    let channels_manager = ChannelsManager::new();
    let channels_list = channels_manager.get_channels_list(deps.storage, start_after, limit)?;
    let channels = channels_list
        .iter()
        .map(|channel| query_channel(deps, channel.channel_id.clone()))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(channels)
}

fn query_playlist(
    deps: Deps,
    channel_id: String,
    playlist_name: String,
) -> Result<Playlist, ContractError> {
    let playlists_manager = PlaylistsManager::new();
    let playlist =
        playlists_manager.get_playlist(deps.storage, channel_id.clone(), playlist_name.clone())?;
    Ok(playlist)
}

fn query_playlists(
    deps: Deps,
    channel_id: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<Playlist>, ContractError> {
    let playlists_manager = PlaylistsManager::new();
    let playlists = playlists_manager.get_all_playlists(
        deps.storage,
        channel_id.clone(),
        start_after,
        limit,
    )?;
    Ok(playlists)
}

fn query_channel_id(deps: Deps, user_name: String) -> Result<String, ContractError> {
    let channels = ChannelsManager::new();
    let channel_id = channels.get_channel_id(deps.storage, user_name.clone())?;
    Ok(channel_id)
}

fn query_is_paused(deps: Deps) -> Result<bool, ContractError> {
    let pause_state = PauseState::new()?;
    let is_paused = pause_state.is_paused(deps.storage)?;
    Ok(is_paused)
}

fn query_pausers(deps: Deps) -> Result<Vec<String>, ContractError> {
    let pause_state = PauseState::new()?;
    let pausers = pause_state.get_pausers(deps.storage)?;
    let pauser_strings = pausers.iter().map(|addr| addr.to_string()).collect();
    Ok(pauser_strings)
}

fn query_assets(
    deps: Deps,
    channel_id: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<AssetResponse>, ContractError> {
    let assets_manager = AssetsManager::new();
    let assets_list =
        assets_manager.get_all_assets(deps.storage, channel_id.clone(), start_after, limit)?;
    Ok(assets_list)
}

fn query_asset(
    deps: Deps,
    channel_id: String,
    publish_id: String,
) -> Result<AssetResponse, ContractError> {
    let assets_manager = AssetsManager::new();
    let asset_key = (channel_id.clone(), publish_id.clone());
    let asset = assets_manager.get_asset(deps.storage, asset_key)?;
    let flags = assets_manager.get_all_flags_for_asset(deps.storage, channel_id, publish_id)?;

    Ok(AssetResponse { asset, flags })
}
fn query_reserved_usernames(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<ReservedUsername>, ContractError> {
    let channels = ChannelsManager::new();
    let reserved_usernames = channels.get_reserved_usernames(deps.storage, start_after, limit)?;
    Ok(reserved_usernames)
}

fn query_channel_collaborator(
    deps: Deps,
    channel_id: String,
    collaborator_address: Addr,
) -> Result<CollaboratorInfo, ContractError> {
    let channels = ChannelsManager::new();
    let collaborator =
        channels.get_collaborator(deps.storage, channel_id, collaborator_address.clone())?;
    Ok(CollaboratorInfo {
        address: collaborator_address.to_string(),
        role: collaborator.role.to_string(),
        share: collaborator.share,
    })
}

fn query_channel_collaborators(
    deps: Deps,
    channel_id: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<CollaboratorInfo>, ContractError> {
    let channels = ChannelsManager::new();
    let collaborators =
        channels.get_channel_collaborators(deps.storage, channel_id, start_after, limit)?;
    Ok(collaborators)
}

fn query_followers_count(deps: Deps, channel_id: String) -> Result<u64, ContractError> {
    let channels = ChannelsManager::new();
    let count = channels.get_followers_count(deps.storage, channel_id)?;
    Ok(count)
}

fn query_followers(
    deps: Deps,
    channel_id: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<Addr>, ContractError> {
    let channels = ChannelsManager::new();
    let followers = channels.get_followers(deps.storage, channel_id, start_after, limit)?;
    Ok(followers)
}

#[cfg(test)]
mod tests {}
