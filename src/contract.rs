use crate::channels::{ChannelDetails, ChannelOnftData, Channels};
use crate::error::ContractError;
use crate::helpers::{
    generate_random_id_with_prefix, get_collection_creation_fee, get_onft_with_owner,
};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::pauser::PauseState;
use crate::playlist::{Asset, Playlists};
use crate::state::CONFIG;
use crate::state::{ChannelConractConfig, CHANNELS_COLLECTION_ID};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure_eq, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
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
        .addr_validate(&msg.fee_collector.clone().into_string())
        .unwrap_or(admin.clone());

    // Save channel CONFIG
    let channel_contract_config = ChannelConractConfig {
        admin: admin.clone(),
        fee_collector: fee_collector.clone(),
        channels_collection_id: msg.channels_collection_id.clone(),
        channels_collection_name: "Channels".to_string(),
        channels_collection_symbol: "CH".to_string(),
        channel_creation_fee: msg.channel_creation_fee.clone(),
    };
    // Save the channel CONFIG to the contract state
    CONFIG.save(deps.storage, &channel_contract_config)?;

    let collection_creation_fee = get_collection_creation_fee(deps.as_ref());

    // Check if the payment provided in the message matches the required creation fee
    ensure_eq!(
        info.funds.clone(),
        vec![collection_creation_fee.clone()],
        ContractError::PaymentError {
            expected: vec![collection_creation_fee.clone()],
            received: info.funds.clone()
        }
    );

    // Prepare the message to create a new ONFT denom (collection)
    let onft_creation_message: CosmosMsg =
        omniflix_std::types::omniflix::onft::v1beta1::MsgCreateDenom {
            id: msg.channels_collection_id.clone(),
            name: msg.channels_collection_name.clone(),
            symbol: msg.channels_collection_symbol.clone(),
            creation_fee: Some(collection_creation_fee.into()),
            ..Default::default()
        }
        .into();

    // Build and return the response with the necessary messages and attributes
    let response = Response::new()
        .add_message(onft_creation_message)
        .add_attribute("action", "instantiate");

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
            playlist_id,
        } => publish(
            deps,
            env,
            info,
            asset_onft_collection_id,
            asset_onft_id,
            salt,
            channel_id,
            playlist_id,
        ),
        ExecuteMsg::CreatePlaylist {
            playlist_id,
            channel_id,
        } => create_playlist(deps, env, info, channel_id, playlist_id),
        ExecuteMsg::RegisterChannel {
            user_name,
            salt,
            description,
        } => register_channel(deps, env, info, salt, description, user_name),
        ExecuteMsg::SetChannelDetails {
            channel_id,
            description,
        } => set_channel_details(deps, info, channel_id, description),
    }
}

fn register_channel(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    salt: Binary,
    description: String,
    user_name: String,
) -> Result<Response, ContractError> {
    let pause_state = PauseState::new()?;
    pause_state.error_if_paused(deps.storage)?;

    // Generate a random channel onft ID
    let onft_id = generate_random_id_with_prefix(&salt, &env, "onft");

    // Generate a random channel ID
    let channel_id = generate_random_id_with_prefix(&salt, &env, "channel");

    let mut channels = Channels::new(deps.storage);

    let channel_details = ChannelDetails::new(
        channel_id.clone(),
        user_name.clone(),
        onft_id.clone(),
        description.clone(),
    );
    channel_details.clone().validate_channel_details()?;

    // Add the new channel to the collection
    // Checks for uniqueness of the channel ID and username
    channels.add_channel(
        channel_id.clone(),
        info.sender.clone().to_string(),
        onft_id.clone(),
        channel_details.clone(),
    )?;

    // Initilize new playlist
    let mut playlists = Playlists::new(deps.storage);
    playlists.initilize_playlist_for_new_channel(channel_id.clone());

    let onft_data = ChannelOnftData {
        channel_id: channel_id.clone(),
        user_name: user_name.clone(),
    };

    let string_onft_data =
        serde_json::to_string(&onft_data).map_err(|_| ContractError::InvalidOnftData {})?;

    let mint_onft_msg: CosmosMsg = omniflix_std::types::omniflix::onft::v1beta1::MsgMintOnft {
        id: onft_id.clone(),
        denom_id: CHANNELS_COLLECTION_ID.load(deps.storage)?,
        sender: env.contract.address.clone().to_string(),
        recipient: info.sender.clone().to_string(),
        data: string_onft_data,
        ..Default::default()
    }
    .into();

    let response = Response::new()
        .add_message(mint_onft_msg)
        .add_attribute("action", "register_channel");
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
    playlist_id: Option<String>,
) -> Result<Response, ContractError> {
    let pause_state = PauseState::new()?;
    pause_state.error_if_paused(deps.storage)?;

    // Find and validate the channel being published to is owned by the sender
    let channels = Channels::new(deps.storage);
    let channel_details = channels.get_channel_details(channel_id.clone())?;
    let channel_onft_id = channel_details.onft_id;
    let channels_collection_id = CHANNELS_COLLECTION_ID.load(deps.storage)?;

    let channel_onft = get_onft_with_owner(
        deps.as_ref(),
        channels_collection_id.clone(),
        channel_onft_id,
        info.sender.clone().to_string(),
    )?;

    // Find and validate the asset being published
    let asset_onft = get_onft_with_owner(
        deps.as_ref(),
        asset_onft_collection_id.clone(),
        asset_onft_id.clone(),
        info.sender.clone().to_string(),
    )?;

    let publish_id = generate_random_id_with_prefix(&salt, &env, "publish");

    let asset = Asset {
        publish_id: publish_id.clone(),
        collection_id: asset_onft_collection_id.clone(),
        onft_id: asset_onft_id.clone(),
    };

    let mut playlists = Playlists::new(deps.storage);
    playlists.add_asset_to_playlist(channel_id.clone(), "My Videos".to_string(), asset.clone())?;

    if let Some(playlist_id) = playlist_id.clone() {
        playlists.add_asset_to_playlist(channel_id.clone(), playlist_id.clone(), asset.clone())?;
    }

    let response = Response::new()
        .add_attribute("action", "publish")
        .add_attribute("publish_id", publish_id)
        .add_attribute("channel_id", channel_id)
        .add_attribute(
            "playlist_id",
            playlist_id.unwrap_or("My Videos".to_string()),
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
    playlist_id: String,
) -> Result<Response, ContractError> {
    let pause_state = PauseState::new()?;
    pause_state.error_if_paused(deps.storage)?;

    // Find and validate the channel being published to is owned by the sender
    let channels = Channels::new(deps.storage);
    let channel_details = channels.get_channel_details(channel_id.clone())?;
    let channel_onft_id = channel_details.onft_id;
    let channels_collection_id = CHANNELS_COLLECTION_ID.load(deps.storage)?;

    let _channel_onft = get_onft_with_owner(
        deps.as_ref(),
        channels_collection_id.clone(),
        channel_onft_id,
        info.sender.clone().to_string(),
    )?;

    let mut playlists = Playlists::new(deps.storage);
    playlists.add_new_playlist(channel_id.clone(), playlist_id.clone())?;

    let response = Response::new()
        .add_attribute("action", "create_playlist")
        .add_attribute("channel_id", channel_id)
        .add_attribute("playlist_id", playlist_id);

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
    // First, handle pause state and immutable querying
    let pause_state = PauseState::new()?;
    pause_state.error_if_paused(deps.storage)?;

    let channels_collection_id = CHANNELS_COLLECTION_ID.load(deps.storage)?;

    let channels = Channels::new(deps.storage);
    let channel_details = channels.get_channel_details(channel_id.clone())?;
    let channel_onft_id = channel_details.onft_id.clone();

    let _channel_onft = get_onft_with_owner(
        deps.as_ref(),
        channels_collection_id.clone(),
        channel_onft_id,
        info.sender.to_string(),
    )?;

    let mut channels = Channels::new(deps.storage); // Re-borrow mutably
    let mut channel_details = channels.get_channel_details(channel_id.clone())?;

    // Update and validate the details
    channel_details.description = description.clone();
    channel_details.validate_channel_details()?;

    // Save updated channel details
    channels.set_channel_details(channel_id.clone(), channel_details)?;

    let response = Response::new()
        .add_attribute("action", "set_channel_details")
        .add_attribute("channel_id", channel_id)
        .add_attribute("description", description);

    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: DepsMut, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::IsPaused {} => todo!(),
        QueryMsg::Pausers {} => todo!(),
        QueryMsg::ChannelDetails { channel_id } => todo!(),
        QueryMsg::Playlist {
            channel_id,
            playlist_id,
        } => todo!(),
        QueryMsg::Channels { start_after, limit } => todo!(),
        QueryMsg::ChannelId { user_name } => todo!(),
        QueryMsg::ChannelOwner { channel_id } => todo!(),
    }
}

fn query_channel_details(
    deps: DepsMut,
    channel_id: String,
) -> Result<ChannelDetails, ContractError> {
    let channels = Channels::new(deps.storage);
    let channel_details = channels.get_channel_details(channel_id.clone())?;
    Ok(channel_details)
}

fn query_channels(
    deps: DepsMut,
    start_after: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<ChannelDetails>, ContractError> {
    let channels = Channels::new(deps.storage);
    let channels_list = channels.get_channels_list(start_after, limit)?;
    Ok(channels_list)
}

#[cfg(test)]
mod tests {}
