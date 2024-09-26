use crate::error::ContractError;
use crate::helpers::{
    bank_msg_wrapper, check_payment, generate_random_id_with_prefix, get_collection_creation_fee,
    get_onft_with_owner,
};
use crate::state::ChannelConractConfig;
use crate::state::CONFIG;
use asset_manager::assets::Assets;
use asset_manager::playlist::PlaylistsManager;
use asset_manager::types::{Asset, Playlist};
use channel_manager::channel::ChannelsManager;
use channel_manager::types::{ChannelDetails, ChannelOnftData};
use channel_types::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult,
};
use pauser::PauseState;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Initialize the pause state and set the initial pausers
    let pause_state = PauseState::new()?;
    pause_state.set_pausers(deps.storage, info.sender.clone(), vec![msg.admin.clone()])?;

    // Validate the admin address provided in the instantiation message
    let admin = deps.api.addr_validate(&msg.clone().admin.into_string())?;

    // Validate the fee collector address, or default to the admin address if validation fails
    let fee_collector = deps
        .api
        .addr_validate(&msg.fee_collector.clone().into_string())?;

    // Save channel CONFIG
    let channel_contract_config = ChannelConractConfig {
        admin: admin.clone(),
        fee_collector: fee_collector.clone(),
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
        .add_attribute("admin", admin.clone().to_string())
        .add_attribute("fee_collector", fee_collector.clone().to_string())
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
        ExecuteMsg::Pause {} => pause(deps, info),
        ExecuteMsg::Unpause {} => unpause(deps, info),
        ExecuteMsg::SetPausers { pausers } => set_pausers(deps, info, pausers),
        ExecuteMsg::Publish {
            asset_onft_collection_id,
            asset_onft_id,
            salt,
            channel_id,
            playlist_name,
            is_visible,
        } => publish(
            deps,
            env,
            info,
            asset_onft_collection_id,
            asset_onft_id,
            salt,
            channel_id,
            playlist_name,
            is_visible,
        ),
        ExecuteMsg::CreatePlaylist {
            playlist_name,
            channel_id,
        } => create_playlist(deps, env, info, channel_id, playlist_name),
        ExecuteMsg::CreateChannel {
            user_name,
            salt,
            description,
            collabarators,
        } => create_channel(deps, env, info, salt, description, user_name, collabarators),
        ExecuteMsg::SetChannelDetails {
            channel_id,
            description,
        } => set_channel_details(deps, info, channel_id, description),
        ExecuteMsg::RemovePlaylist {
            playlist_name,
            channel_id,
        } => remove_playlist(deps, info, channel_id, playlist_name),
        ExecuteMsg::RemoveAsset {
            publish_id,
            channel_id,
            playlist_name,
        } => remove_asset(deps, info, publish_id, channel_id, playlist_name),
        ExecuteMsg::SetConfig {
            channel_creation_fee,
            admin,
            fee_collector,
        } => set_config(deps, info, channel_creation_fee, admin, fee_collector),
        ExecuteMsg::AddAsset {
            publish_id,
            asset_channel_id,
            channel_id,
            playlist_name,
        } => add_asset(
            deps,
            info,
            asset_channel_id,
            publish_id,
            channel_id,
            playlist_name,
        ),
    }
}

fn create_channel(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    salt: Binary,
    description: String,
    user_name: String,
    collabarators: Option<Vec<String>>,
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

    // Validate the collabarators addresses
    // If no collabarators are provided, the vector will be empty
    let addr_collaborators: Vec<Addr> = collabarators
        .clone()
        .unwrap_or_default()
        .iter()
        .map(|collaborator| deps.api.addr_validate(collaborator))
        .collect::<Result<Vec<Addr>, _>>()?;

    let channel_details = ChannelDetails::new(
        channel_id.clone(),
        user_name.clone(),
        description.clone(),
        onft_id.clone(),
        addr_collaborators,
    );
    channel_details.clone().validate()?;

    // Add the new channel to the collection
    // Checks for uniqueness of the channel ID and username
    channels_manager.add_channel(
        deps.storage,
        channel_id.clone(),
        user_name.clone(),
        channel_details.clone(),
    )?;

    // Create the onft data for the channel. This data will be stored in the onft's data field
    let onft_data = ChannelOnftData {
        channel_id: channel_id.clone(),
        user_name: user_name.clone(),
        onft_id: onft_id.clone(),
    };

    let string_onft_data =
        serde_json::to_string(&onft_data).map_err(|_| ContractError::InvalidOnftData {})?;

    let mint_onft_msg: CosmosMsg = omniflix_std::types::omniflix::onft::v1beta1::MsgMintOnft {
        id: onft_id.clone(),
        denom_id: config.channels_collection_id.clone(),
        sender: env.contract.address.clone().to_string(),
        recipient: info.sender.clone().to_string(),
        data: string_onft_data,
        ..Default::default()
    }
    .into();

    // Pay the channel creation fee to the fee collector
    let bank_channel_fee_msg = bank_msg_wrapper(
        config.fee_collector.into_string(),
        config.channel_creation_fee,
    );

    let response = Response::new()
        .add_message(mint_onft_msg)
        .add_messages(bank_channel_fee_msg)
        .add_attribute("action", "register_channel")
        .add_attribute("channel_id", channel_id)
        .add_attribute("user_name", user_name)
        .add_attribute("onft_id", onft_id);
    Ok(response)
}
fn publish(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    asset_onft_collection_id: String,
    asset_onft_id: String,
    salt: Binary,
    channel_id: String,
    playlist_name: Option<String>,
    is_visible: bool,
) -> Result<Response, ContractError> {
    let pause_state = PauseState::new()?;
    pause_state.error_if_paused(deps.storage)?;

    let config = CONFIG.load(deps.storage)?;

    // Find and validate the channel being published to is owned by the sender
    // or the sender is a collaborator
    let channels = ChannelsManager::new();
    let channel_details = channels.get_channel_details(deps.storage, channel_id.clone())?;
    let channel_onft_id = channel_details.onft_id;
    // Check if the sender is a collaborator
    // Else check if the sender is the owner
    if !channel_details.collabarators.contains(&info.sender) {
        let _channel_onft = get_onft_with_owner(
            deps.as_ref(),
            config.channels_collection_id.clone(),
            channel_onft_id,
            info.sender.clone().to_string(),
        )?;
    };

    // Find and validate the asset being published
    let _asset_onft = get_onft_with_owner(
        deps.as_ref(),
        asset_onft_collection_id.clone(),
        asset_onft_id.clone(),
        info.sender.clone().to_string(),
    )?;

    let publish_id = generate_random_id_with_prefix(&salt, &env, "publish");
    // Define the asset to be published
    let asset = Asset {
        channel_id: channel_id.clone(),
        publish_id: publish_id.clone(),
        collection_id: asset_onft_collection_id.clone(),
        onft_id: asset_onft_id.clone(),
        is_visible: is_visible,
    };
    // Add asset to the channel's asset list
    let assets = Assets::new();
    assets.add_asset(deps.storage, channel_id.clone(), asset.clone())?;

    let response = Response::new()
        .add_attribute("action", "publish")
        .add_attribute("publish_id", publish_id)
        .add_attribute("channel_id", channel_id)
        .add_attribute(
            "playlist_name",
            playlist_name.unwrap_or("My Videos".to_string()),
        )
        .add_attribute("asset_onft_collection_id", asset_onft_collection_id)
        .add_attribute("asset_onft_id", asset_onft_id);

    Ok(response)
}

fn create_playlist(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    channel_id: String,
    playlist_name: String,
) -> Result<Response, ContractError> {
    let pause_state = PauseState::new()?;
    pause_state.error_if_paused(deps.storage)?;

    let config = CONFIG.load(deps.storage)?;

    // Find and validate the channel being published to is owned by the sender
    let channel_manager = ChannelsManager::new();
    let channel_details = channel_manager.get_channel_details(deps.storage, channel_id.clone())?;
    let channel_onft_id = channel_details.onft_id;

    let _channel_onft = get_onft_with_owner(
        deps.as_ref(),
        config.channels_collection_id.clone(),
        channel_onft_id,
        info.sender.clone().to_string(),
    )?;

    let playlist_manager = PlaylistsManager::new();
    playlist_manager.add_new_playlist(deps.storage, channel_id.clone(), playlist_name.clone())?;

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

fn set_channel_details(
    deps: DepsMut,
    info: MessageInfo,
    channel_id: String,
    description: String,
) -> Result<Response, ContractError> {
    let pause_state = PauseState::new()?;
    pause_state.error_if_paused(deps.storage)?;

    let config = CONFIG.load(deps.storage)?;

    let channels_manager = ChannelsManager::new();
    let mut channel_details =
        channels_manager.get_channel_details(deps.storage, channel_id.clone())?;
    let channel_onft_id = channel_details.onft_id.clone();

    let _channel_onft = get_onft_with_owner(
        deps.as_ref(),
        config.channels_collection_id.clone(),
        channel_onft_id,
        info.sender.to_string(),
    )?;

    channel_details.description = description.clone();
    channel_details.validate()?;

    channels_manager.update_channel_details(
        deps.storage,
        channel_id.clone(),
        channel_details.clone(),
    )?;

    let response = Response::new()
        .add_attribute("action", "set_channel_details")
        .add_attribute("channel_id", channel_id)
        .add_attribute("description", description);

    Ok(response)
}

fn remove_playlist(
    deps: DepsMut,
    info: MessageInfo,
    channel_id: String,
    playlist_name: String,
) -> Result<Response, ContractError> {
    let pause_state = PauseState::new()?;
    pause_state.error_if_paused(deps.storage)?;

    let config = CONFIG.load(deps.storage)?;

    let channels = ChannelsManager::new();
    let channel_details = channels.get_channel_details(deps.storage, channel_id.clone())?;
    let channel_onft_id = channel_details.onft_id;

    let _channel_onft = get_onft_with_owner(
        deps.as_ref(),
        config.channels_collection_id.clone(),
        channel_onft_id,
        info.sender.clone().to_string(),
    )?;

    let playlist_manager = PlaylistsManager::new();
    playlist_manager.remove_playlist(deps.storage, channel_id.clone(), playlist_name.clone())?;

    let response = Response::new()
        .add_attribute("action", "remove_playlist")
        .add_attribute("channel_id", channel_id)
        .add_attribute("playlist_name", playlist_name);

    Ok(response)
}
fn add_asset(
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

    let channels = ChannelsManager::new();
    let channel_details = channels.get_channel_details(deps.storage, channel_id.clone())?;
    let channel_onft_id = channel_details.onft_id;

    if !channel_details.collabarators.contains(&info.sender) {
        let _channel_onft = get_onft_with_owner(
            deps.as_ref(),
            config.channels_collection_id.clone(),
            channel_onft_id,
            info.sender.clone().to_string(),
        )?;
    };

    let playlist_manager = PlaylistsManager::new();

    // Load the asset
    let assets = Assets::new();
    let asset = assets.get_asset(deps.storage, asset_channel_id.clone(), publish_id.clone())?;

    // Verify that the asset is visible
    if asset.is_visible == false {
        return Err(ContractError::AssetNotVisible {});
    }
    // Add asset to playlist
    playlist_manager.add_asset_to_playlist(
        deps.storage,
        channel_id.clone(),
        playlist_name.clone(),
        asset.clone(),
    )?;

    let response = Response::new()
        .add_attribute("action", "add_asset")
        .add_attribute("channel_id", channel_id)
        .add_attribute("playlist_name", playlist_name)
        .add_attribute("publish_id", publish_id);

    Ok(response)
}
fn remove_asset(
    deps: DepsMut,
    info: MessageInfo,
    publish_id: String,
    channel_id: String,
    playlist_name: String,
) -> Result<Response, ContractError> {
    let pause_state = PauseState::new()?;
    pause_state.error_if_paused(deps.storage)?;

    let config = CONFIG.load(deps.storage)?;

    let channels = ChannelsManager::new();
    let channel_details = channels.get_channel_details(deps.storage, channel_id.clone())?;
    let channel_onft_id = channel_details.onft_id;

    let _channel_onft = get_onft_with_owner(
        deps.as_ref(),
        config.channels_collection_id.clone(),
        channel_onft_id,
        info.sender.clone().to_string(),
    )?;

    let playlist_manager = PlaylistsManager::new();
    // Remove the asset from the playlist
    playlist_manager.remove_asset_from_playlist(
        deps.storage,
        channel_id.clone(),
        playlist_name.clone(),
        publish_id.clone(),
    )?;

    let response = Response::new()
        .add_attribute("action", "remove_asset")
        .add_attribute("channel_id", channel_id)
        .add_attribute("playlist_name", playlist_name)
        .add_attribute("publish_id", publish_id);

    Ok(response)
}

fn set_config(
    deps: DepsMut,
    info: MessageInfo,
    channel_creation_fee: Option<Vec<Coin>>,
    admin: Option<String>,
    fee_collector: Option<String>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(admin) = admin {
        let admin = deps.api.addr_validate(&admin)?;
        config.admin = admin;
    }

    if let Some(fee_collector) = fee_collector {
        let fee_collector = deps.api.addr_validate(&fee_collector)?;
        config.fee_collector = fee_collector;
    }

    if let Some(channel_creation_fee) = channel_creation_fee {
        config.channel_creation_fee = channel_creation_fee;
    }

    CONFIG.save(deps.storage, &config)?;

    let response = Response::new()
        .add_attribute("action", "set_config")
        .add_attribute("admin", config.admin.to_string())
        .add_attribute("fee_collector", config.fee_collector.to_string());

    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::IsPaused {} => to_json_binary(&query_is_paused(deps)?),
        QueryMsg::Pausers {} => to_json_binary(&query_pausers(deps)?),
        QueryMsg::ChannelDetails {
            channel_id,
            user_name,
        } => to_json_binary(&query_channel_details(deps, channel_id, user_name)?),
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
    }
}

fn query_channel_details(
    deps: Deps,
    channel_id: Option<String>,
    user_name: Option<String>,
) -> Result<ChannelDetails, ContractError> {
    let channels = ChannelsManager::new();
    // Match channels Id and user name
    let channel_details = match (channel_id, user_name) {
        (Some(channel_id), None) => channels.get_channel_details(deps.storage, channel_id)?,
        (None, Some(user_name)) => {
            let channel_id = channels.get_channel_id(deps.storage, user_name)?;
            channels.get_channel_details(deps.storage, channel_id)?
        }
        _ => return Err(ContractError::InvalidChannelQuery {}),
    };
    Ok(channel_details)
}

fn query_channels(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<ChannelDetails>, ContractError> {
    let channels = ChannelsManager::new();
    let channels_list = channels.get_channels_list(deps.storage, start_after, limit)?;
    Ok(channels_list)
}

fn query_playlist(
    deps: Deps,
    channel_id: String,
    playlist_name: String,
) -> Result<Playlist, ContractError> {
    let playlists = PlaylistsManager::new();
    let playlist =
        playlists.get_playlist(deps.storage, channel_id.clone(), playlist_name.clone())?;
    Ok(playlist)
}

fn query_playlists(
    deps: Deps,
    channel_id: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<Playlist>, ContractError> {
    let playlists = PlaylistsManager::new();
    let playlists_list =
        playlists.get_all_playlists(deps.storage, channel_id.clone(), start_after, limit)?;
    Ok(playlists_list)
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

fn query_pausers(deps: Deps) -> Result<Vec<Addr>, ContractError> {
    let pause_state = PauseState::new()?;
    let pausers = pause_state.get_pausers(deps.storage)?;
    Ok(pausers)
}

#[cfg(test)]
mod tests {}
