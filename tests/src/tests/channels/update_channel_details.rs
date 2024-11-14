use crate::helpers::setup::setup;
use channel_manager::error::ChannelError;
use channel_manager::types::ChannelDetails;
use channel_types::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cosmwasm_std::{coin, Binary};
use cw_multi_test::Executor;
use omniflix_channel::ContractError;

#[test]
fn missing_channel_id() {
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

    // Missing channel_id
    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::ChannelUpdateDetails {
                channel_id: "".to_string(),
                description: "creator".to_string(),
            },
            &[],
        )
        .unwrap_err();

    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::Channel(channel_manager::error::ChannelError::ChannelIdNotFound {})
    );
}

#[test]
fn missing_description() {
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
    // Create a channel
    let _res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::ChannelCreate {
                salt: Binary::default(),
                user_name: "creator".to_string(),
                description: "creator".to_string(),
                collaborators: None,
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap();

    // Query Channel
    let channel: ChannelDetails = app
        .wrap()
        .query_wasm_smart(
            channel_contract_addr.clone(),
            &QueryMsg::ChannelDetails {
                channel_id: None,
                user_name: Some("creator".to_string()),
            },
        )
        .unwrap();
    let channel_id = channel.channel_id.clone();

    // Missing description
    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::ChannelUpdateDetails {
                channel_id: channel_id.clone(),
                description: "".to_string(),
            },
            &[],
        )
        .unwrap_err();

    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::Channel(ChannelError::InvalidDescription {})
    );
}

#[test]
fn invalid_channel() {
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

    // Channel not found
    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::ChannelUpdateDetails {
                channel_id: "1".to_string(),
                description: "creator".to_string(),
            },
            &[],
        )
        .unwrap_err();

    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::Channel(ChannelError::ChannelIdNotFound {})
    );
}

#[test]
fn unauthorized() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();
    let creator = setup_response.test_accounts.creator.clone();
    let collector = setup_response.test_accounts.collector.clone();

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

    let creator_flix_balance = app
        .wrap()
        .query_balance(creator.clone(), "uflix")
        .unwrap()
        .amount;
    println!("Creator Flix Balance: {:?}", creator_flix_balance);

    // Create a channel
    let _res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::ChannelCreate {
                salt: Binary::default(),
                user_name: "creator".to_string(),
                description: "creator".to_string(),
                collaborators: None,
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap();

    // Query Channel
    let channel: ChannelDetails = app
        .wrap()
        .query_wasm_smart(
            channel_contract_addr.clone(),
            &QueryMsg::ChannelDetails {
                channel_id: None,
                user_name: Some("creator".to_string()),
            },
        )
        .unwrap();
    let channel_id = channel.channel_id.clone();
    let onft_id = channel.onft_id.clone();

    // Unauthorized
    let res = app
        .execute_contract(
            collector.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::ChannelUpdateDetails {
                channel_id: channel_id.clone(),
                description: "creator".to_string(),
            },
            &[],
        )
        .unwrap_err();

    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::OnftNotOwned {
            collection_id: "Channels".to_string(),
            onft_id: onft_id.clone()
        }
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

    // Create a channel
    let _res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::ChannelCreate {
                salt: Binary::default(),
                user_name: "creator".to_string(),
                description: "creator".to_string(),
                collaborators: None,
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap();

    // Query Channel
    let channel: ChannelDetails = app
        .wrap()
        .query_wasm_smart(
            channel_contract_addr.clone(),
            &QueryMsg::ChannelDetails {
                channel_id: None,
                user_name: Some("creator".to_string()),
            },
        )
        .unwrap();
    let channel_id = channel.channel_id.clone();

    // Happy path
    let _res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::ChannelUpdateDetails {
                channel_id: channel_id.clone(),
                description: "new description".to_string(),
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap();

    // Query Channel
    let channel: ChannelDetails = app
        .wrap()
        .query_wasm_smart(
            channel_contract_addr.clone(),
            &QueryMsg::ChannelDetails {
                channel_id: Some(channel_id.clone()),
                user_name: None,
            },
        )
        .unwrap();
    assert_eq!(channel.description, "new description");
}
