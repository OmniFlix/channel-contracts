use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    ADMIN, FINAL_RANDOMNESS, NOIS_PROXY, PARTICIPANT_COUNT, TEST_RANDOMNESS, TEST_WINNERS, WINNERS,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure_eq, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    WasmMsg,
};
use nois::pick;
use nois::{NoisCallback, ProxyExecuteMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let nois_proxy_addr = deps
        .api
        .addr_validate(&msg.nois_proxy_address)
        .map_err(|_| ContractError::InvalidProxyAddress {})?;
    if msg.participant_count == 0 {
        return Err(ContractError::InvalidParticipantCount {});
    }
    PARTICIPANT_COUNT.save(deps.storage, &msg.participant_count)?;
    NOIS_PROXY.save(deps.storage, &nois_proxy_addr)?;
    ADMIN.save(deps.storage, &info.sender)?;

    let res = Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("participant_count", msg.participant_count.to_string())
        .add_attribute("nois_proxy_address", msg.nois_proxy_address)
        .add_attribute("nois_proxy_addr", nois_proxy_addr.to_string())
        .add_attribute("creator", info.sender.to_string());

    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::RequestRandomness {
            job_id,
            delay_in_mins,
        } => execute_request_randomness(deps, env, info, job_id, delay_in_mins),
        ExecuteMsg::NoisReceive { callback } => execute_set_randomness(deps, env, info, callback),
        ExecuteMsg::PickTestWinners {} => pick_test_winners(deps, env, info),
        ExecuteMsg::PickWinners {} => pick_winners(deps, env, info),
    }
}

pub fn execute_request_randomness(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    job_id: String,
    delay_in_mins: u64,
) -> Result<Response, ContractError> {
    let nois_proxy = NOIS_PROXY.load(deps.storage)?;

    if info.sender != ADMIN.load(deps.storage)? {
        return Err(ContractError::Unauthorized {});
    }
    let now = env.block.time;
    let res = Response::new()
        .add_message(WasmMsg::Execute {
            contract_addr: nois_proxy.into(),
            msg: to_json_binary(&ProxyExecuteMsg::GetRandomnessAfter {
                after: now.plus_minutes(delay_in_mins),
                job_id: job_id.clone(),
            })?,
            funds: info.funds.clone(),
        })
        .add_attribute("action", "request_randomness")
        .add_attribute("job_id", job_id)
        .add_attribute("after", now.plus_minutes(delay_in_mins).to_string());
    Ok(res)
}

pub fn execute_set_randomness(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    callback: NoisCallback,
) -> Result<Response, ContractError> {
    let proxy = NOIS_PROXY.load(deps.storage)?;
    ensure_eq!(info.sender, proxy, ContractError::Unauthorized {});

    let NoisCallback {
        job_id, randomness, ..
    } = callback;

    if job_id.contains("test") {
        TEST_RANDOMNESS.save(deps.storage, &randomness)?;
    } else {
        if FINAL_RANDOMNESS.may_load(deps.storage)?.is_some() {
            return Err(ContractError::FinalRandomnessAlreadySet {});
        }
        FINAL_RANDOMNESS.save(deps.storage, &randomness)?;
    }

    let res = Response::new();
    Ok(res)
}

pub fn pick_test_winners(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
) -> Result<Response, ContractError> {
    let participant_count = PARTICIPANT_COUNT.load(deps.storage)?;
    let participants = (1..=participant_count).collect::<Vec<u32>>();
    let test_randomness = TEST_RANDOMNESS.load(deps.storage)?;
    let test_randomness_vec: [u8; 32] = test_randomness[..].try_into().unwrap();

    if test_randomness.is_empty() {
        return Err(ContractError::TestRandomnessNotSet {});
    }

    let winners = pick(test_randomness_vec, 100, participants);
    let winners_str = winners
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(",");
    TEST_WINNERS.save(deps.storage, &winners)?;

    let res = Response::new()
        .add_attribute("action", "pick_test_winners")
        .add_attribute("test winners", winners_str);
    Ok(res)
}

pub fn pick_winners(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    if info.sender != ADMIN.load(deps.storage)? {
        return Err(ContractError::Unauthorized {});
    }
    if WINNERS.may_load(deps.storage)?.is_some() {
        return Err(ContractError::WinnersAlreadyPicked {});
    }

    let participant_count = PARTICIPANT_COUNT.load(deps.storage)?;
    let participants = (1..=participant_count).collect::<Vec<u32>>();

    let final_randomness = FINAL_RANDOMNESS
        .load(deps.storage)
        .map_err(|_| ContractError::FinalRandomnessNotSet {})?;
    let final_randomness_vec: [u8; 32] = final_randomness[..].try_into().unwrap();

    let winners = pick(final_randomness_vec, 100, participants);
    let winners_str = winners
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(",");

    WINNERS.save(deps.storage, &winners)?;

    let res = Response::new()
        .add_attribute("action", "pick_winners")
        .add_attribute("winners", winners_str);

    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ParticipantCount {} => to_json_binary(&PARTICIPANT_COUNT.load(_deps.storage)?),
        QueryMsg::Winners {} => to_json_binary(&WINNERS.load(_deps.storage)?),
        QueryMsg::Admin {} => to_json_binary(&ADMIN.load(_deps.storage)?),
        QueryMsg::NoisProxy {} => to_json_binary(&NOIS_PROXY.load(_deps.storage)?),
        QueryMsg::TestWinners {} => to_json_binary(&TEST_WINNERS.load(_deps.storage)?),
    }
}

#[cfg(test)]
mod tests {}
