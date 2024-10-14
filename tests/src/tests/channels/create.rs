use channel_manager::error::ChannelError;
use channel_manager::types::ChannelDetails;
use channel_types::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cosmwasm_std::{coin, Binary};
use cw_multi_test::Executor;
use omniflix_channel::ContractError;

use crate::helpers::setup::setup;
use crate::helpers::utils::get_event_attribute;

#[test]
fn missing_creation_fee() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();
    let creator = setup_response.test_accounts.creator.clone();

    let instantiate_msg = InstantiateMsg {
        admin: setup_response.test_accounts.admin.clone(),
        channel_creation_fee: vec![coin(1000000, "uflix")],
        fee_collector: setup_response.test_accounts.admin,
        channels_collection_id: "Channels".to_string(),
        channels_collection_name: "Channels".to_string(),
        channels_collection_symbol: "CH".to_string(),
    };

    // Instantiate the contract
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

    // Missing creation fee creating a channel
    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::ChannelCreate {
                salt: Binary::default(),
                user_name: "creator".to_string(),
                description: "creator".to_string(),
                collabarators: None,
            },
            &[],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let typed_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::PaymentError {
            expected: [coin(1000000, "uflix")].to_vec(),
            received: (vec![])
        }
    );
}

#[test]
fn paused() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();
    let creator = setup_response.test_accounts.creator.clone();

    let instantiate_msg = InstantiateMsg {
        admin: setup_response.test_accounts.admin.clone(),
        channel_creation_fee: vec![coin(1000000, "uflix")],
        fee_collector: setup_response.test_accounts.admin,
        channels_collection_id: "Channels".to_string(),
        channels_collection_name: "Channels".to_string(),
        channels_collection_symbol: "CH".to_string(),
    };

    // Instantiate the contract
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

    // Pause the contract
    let _res = app
        .execute_contract(
            admin.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::Pause {},
            &[],
        )
        .unwrap();

    // Create a channel
    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::ChannelCreate {
                salt: Binary::default(),
                user_name: "creator".to_string(),
                description: "creator".to_string(),
                collabarators: None,
            },
            &[],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let typed_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::Pause(pauser::PauseError::Paused {})
    );
}

#[test]
fn failed_validations() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();
    let creator = setup_response.test_accounts.creator.clone();

    let instantiate_msg = InstantiateMsg {
        admin: setup_response.test_accounts.admin.clone(),
        channel_creation_fee: vec![coin(1000000, "uflix")],
        fee_collector: setup_response.test_accounts.admin,
        channels_collection_id: "Channels".to_string(),
        channels_collection_name: "Channels".to_string(),
        channels_collection_symbol: "CH".to_string(),
    };

    // Instantiate the contract
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

    // Too long username
    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::ChannelCreate {
                salt: Binary::default(),
                user_name: "creatorcreatorcreatorcreatorcreator".to_string(),
                description: "creator".to_string(),
                collabarators: None,
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let typed_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::Channel(ChannelError::InvalidUserName {})
    );

    // Too long description
    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::ChannelCreate {
                salt: Binary::default(),
                user_name: "creator".to_string(),
                // Generate a sting with 257 characters
                description: "a".repeat(257),
                collabarators: None,
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let typed_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::Channel(ChannelError::InvalidDescription {})
    );
}

#[test]
fn happy_path() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();
    let creator = setup_response.test_accounts.creator.clone();

    let instantiate_msg = InstantiateMsg {
        admin: setup_response.test_accounts.admin.clone(),
        channel_creation_fee: vec![coin(1000000, "uflix")],
        fee_collector: setup_response.test_accounts.admin,
        channels_collection_id: "Channels".to_string(),
        channels_collection_name: "Channels".to_string(),
        channels_collection_symbol: "CH".to_string(),
    };

    // Instantiate the contract
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

    // Happy path
    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::ChannelCreate {
                salt: Binary::default(),
                user_name: "creator".to_string(),
                description: "creator".to_string(),
                collabarators: None,
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap();
    // Validate the creation fee was sent to the fee collector
    let amount = get_event_attribute(res.clone(), "transfer", "amount");
    assert_eq!(amount, "1000000uflix");
    let recipient = get_event_attribute(res.clone(), "transfer", "recipient");
    assert_eq!(recipient, admin.to_string());

    // Get onftid from events
    let onft_id = get_event_attribute(res.clone(), "wasm", "onft_id");

    // Query channels
    let channels: Vec<ChannelDetails> = app
        .wrap()
        .query_wasm_smart(
            channel_contract_addr.clone(),
            &QueryMsg::Channels {
                start_after: None,
                limit: None,
            },
        )
        .unwrap();
    assert_eq!(channels.len(), 1);
    assert_eq!(channels[0].user_name, "creator");
    assert_eq!(channels[0].description, "creator");
    // Validate the onft_id
    assert_eq!(channels[0].onft_id, onft_id);
}
