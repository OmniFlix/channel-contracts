use crate::error::ContractError;
use crate::helpers::get_collection_creation_fee;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::pauser::PauseState;
use crate::state::{ChannelParams, PARAMS};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure_eq, to_json_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, WasmMsg,
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

    // Save channel params
    let channel_params = ChannelParams {
        admin: admin.clone(),
        fee_collector: fee_collector.clone(),
        channels_collection_id: msg.channels_collection_id.clone(),
        channels_collection_name: "Channels".to_string(),
        channels_collection_symbol: "CH".to_string(),
        channel_creation_fee: msg.channel_creation_fee.clone(),
    };
    // Save the channel params to the contract state
    PARAMS.save(deps.storage, &channel_params)?;

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
    match msg {}
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {}
}

#[cfg(test)]
mod tests {}
