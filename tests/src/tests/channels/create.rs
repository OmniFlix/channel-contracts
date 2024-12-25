use channel_manager::types::ChannelDetails;
use cosmwasm_std::{coin, Binary};
use cw_multi_test::Executor;
use omniflix_channel::ContractError;
use omniflix_channel_types::msg::{ExecuteMsg, QueryMsg, ReservedUsername};

use crate::helpers::msg_wrapper::get_channel_instantiate_msg;
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

    let mut instantiate_msg = get_channel_instantiate_msg(admin.clone());
    instantiate_msg.channel_creation_fee = vec![coin(1000000, "uflix")];

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
                collaborators: None,
                banner_picture: None,
                profile_picture: None,
                channel_name: "creator".to_string(),
            },
            &[],
        )
        .unwrap_err();

    let typed_err = res.downcast_ref::<ContractError>().unwrap();
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

    let mut instantiate_msg = get_channel_instantiate_msg(admin.clone());
    instantiate_msg.channel_creation_fee = vec![coin(1000000, "uflix")];

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
                collaborators: None,
                banner_picture: None,
                profile_picture: None,
                channel_name: "creator".to_string(),
            },
            &[],
        )
        .unwrap_err();

    let typed_err = res.downcast_ref::<ContractError>().unwrap();
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

    let mut instantiate_msg = get_channel_instantiate_msg(admin.clone());
    instantiate_msg.channel_creation_fee = vec![coin(1000000, "uflix")];

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
                collaborators: None,
                banner_picture: None,
                profile_picture: None,
                channel_name: "creator".to_string(),
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();

    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(typed_err, &ContractError::InvalidUserName {});

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
                collaborators: None,
                banner_picture: None,
                profile_picture: None,
                channel_name: "creator".to_string(),
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();

    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(typed_err, &ContractError::InvalidDescription {});
}

#[test]
fn happy_path() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();
    let creator = setup_response.test_accounts.creator.clone();

    let mut instantiate_msg = get_channel_instantiate_msg(admin.clone());
    instantiate_msg.channel_creation_fee = vec![coin(1000000, "uflix")];
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
                collaborators: None,
                banner_picture: None,
                profile_picture: None,
                channel_name: "creator".to_string(),
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
    // Validate the onft_id
    assert_eq!(channels[0].onft_id, onft_id);
}

#[test]
fn create_reserved_channel() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();
    let creator = setup_response.test_accounts.creator.clone();

    let mut instantiate_msg = get_channel_instantiate_msg(admin.clone());
    instantiate_msg.channel_creation_fee = vec![coin(1000000, "uflix")];
    // Username "admin" is reserved for the actor admin
    instantiate_msg.reserved_usernames = vec![
        ReservedUsername {
            username: "admin".to_string(),
            address: Some(admin.clone().to_string()),
        },
        ReservedUsername {
            username: "reserved".to_string(),
            address: None,
        },
    ];

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

    // Creator can not use the reserved username "admin"
    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::ChannelCreate {
                salt: Binary::default(),
                user_name: "admin".to_string(),
                description: "creator".to_string(),
                collaborators: None,
                banner_picture: None,
                profile_picture: None,
                channel_name: "creator".to_string(),
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();

    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(typed_err, &ContractError::UserNameReserved {});
    // No one can use the reserved username "reserved"
    let _res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::ChannelCreate {
                salt: Binary::default(),
                user_name: "reserved".to_string(),
                description: "creator".to_string(),
                collaborators: None,
                banner_picture: None,
                profile_picture: None,
                channel_name: "creator".to_string(),
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();
    let _res = app
        .execute_contract(
            admin.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::ChannelCreate {
                salt: Binary::default(),
                user_name: "reserved".to_string(),
                description: "creator".to_string(),
                collaborators: None,
                banner_picture: None,
                profile_picture: None,
                channel_name: "creator".to_string(),
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();

    // Admin can use the reserved username "admin"
    let _res = app
        .execute_contract(
            admin.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::ChannelCreate {
                salt: Binary::default(),
                user_name: "admin".to_string(),
                description: "creator".to_string(),
                collaborators: None,
                banner_picture: None,
                profile_picture: None,
                channel_name: "creator".to_string(),
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap();
    // Whenever a reserved username is used, remove it from the reserved list
    let query_msg = QueryMsg::ReservedUsernames {
        limit: None,
        start_after: None,
    };
    let res: Vec<ReservedUsername> = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].username, "reserved");
}
