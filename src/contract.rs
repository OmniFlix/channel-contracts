use crate::channels::{ChannelDetails, ChannelOnftData, Channels};
use crate::error::ContractError;
use crate::helpers::{generate_random_id_with_prefix, get_collection_creation_fee, get_onft};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::pauser::PauseState;
use crate::playlist::Playlists;
use crate::state::CONFIG;
use crate::state::{ChannelConractConfig, CHANNELS_COLLECTION_ID};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure_eq, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
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
        ExecuteMsg::Pause {} => todo!(),
        ExecuteMsg::Unpause {} => todo!(),
        ExecuteMsg::SetPausers { pausers } => todo!(),
        ExecuteMsg::Publish {
            onft_collection_id,
            onft_id,
            salt,
            channel_id,
            playlist_id,
        } => todo!(),
        ExecuteMsg::CreatePlaylist {
            playlist_id,
            channel_id,
        } => todo!(),
        ExecuteMsg::RegisterChannel { channel_id, salt } => todo!(),
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

    // Add the new channel to the collection
    // Checks for uniqueness of the channel ID and username
    channels.add_channel(
        channel_id.clone(),
        info.sender.clone().to_string(),
        onft_id.clone(),
        description.clone(),
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::IsPaused {} => todo!(),
        QueryMsg::Pausers {} => todo!(),
    }
}

#[cfg(test)]
mod tests {}
