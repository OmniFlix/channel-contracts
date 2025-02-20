use cosmwasm_std::{coin, StdError};
use cw_multi_test::Executor;
use omniflix_channel::ContractError;
use omniflix_channel_types::config::ChannelConractConfig;
use omniflix_channel_types::msg::{ExecuteMsg, QueryMsg};

use crate::helpers::{msg_wrapper::get_channel_instantiate_msg, setup::setup};

#[test]
fn happy_path() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();
    let creator = setup_response.test_accounts.creator.clone();

    // Instantiate the contract
    let instantiate_msg = get_channel_instantiate_msg(admin.clone());
    let channel_contract_addr = app
        .instantiate_contract(
            setup_response.channel_contract_code_id,
            admin.clone(),
            &instantiate_msg,
            &[coin(1000000, "uflix")],
            "Instantiate Channel Contract",
            None,
        )
        .unwrap();

    // Set the config
    let _res = app
        .execute_contract(
            admin.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::AdminSetConfig {
                protocol_admin: Some(creator.to_string()),
                channel_creation_fee: Some(vec![]),
                fee_collector: Some(creator.to_string()),
            },
            &[],
        )
        .unwrap();

    // Check the contract config
    let config: ChannelConractConfig = app
        .wrap()
        .query_wasm_smart(channel_contract_addr, &QueryMsg::Config {})
        .unwrap();
    assert_eq!(config.auth_details.protocol_admin, creator);
}

#[test]
fn unauthorized() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();
    let creator = setup_response.test_accounts.creator.clone();

    // Instantiate the contract
    let instantiate_msg = get_channel_instantiate_msg(admin.clone());
    let channel_contract_addr = app
        .instantiate_contract(
            setup_response.channel_contract_code_id,
            admin.clone(),
            &instantiate_msg,
            &[coin(1000000, "uflix")],
            "Instantiate Channel Contract",
            None,
        )
        .unwrap();

    // Set the config
    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::AdminSetConfig {
                protocol_admin: Some(creator.to_string()),
                channel_creation_fee: Some(vec![]),
                fee_collector: Some(creator.to_string()),
            },
            &[],
        )
        .unwrap_err();

    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(typed_err, &ContractError::Unauthorized {});
}

#[test]
fn invalid() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors

    let admin = setup_response.test_accounts.admin.clone();
    let creator = setup_response.test_accounts.creator.clone();

    // Instantiate the contract
    let instantiate_msg = get_channel_instantiate_msg(admin.clone());
    let channel_contract_addr = app
        .instantiate_contract(
            setup_response.channel_contract_code_id,
            admin.clone(),
            &instantiate_msg,
            &[coin(1000000, "uflix")],
            "Instantiate Channel Contract",
            None,
        )
        .unwrap();

    // Set the config
    let res = app
        .execute_contract(
            admin.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::AdminSetConfig {
                // Invalid address
                protocol_admin: Some("creator".to_string()),
                channel_creation_fee: Some(vec![]),
                fee_collector: Some(creator.to_string()),
            },
            &[],
        )
        .unwrap_err();

    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::Std(StdError::generic_err("Error decoding bech32"))
    );
}
