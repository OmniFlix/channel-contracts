use channel_manager::types::ChannelDetails;
use channel_types::msg::{ExecuteMsg, QueryMsg};
use cosmwasm_std::{coin, Binary};
use cw_multi_test::Executor;
use omniflix_channel::ContractError;

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

    let creator_flix_balance = app
        .wrap()
        .query_balance(creator.clone(), "uflix")
        .unwrap()
        .amount;
    println!("Creator Flix Balance: {:?}", creator_flix_balance);

    let admin_flix_balance = app
        .wrap()
        .query_balance(admin.clone(), "uflix")
        .unwrap()
        .amount;
    println!("Admin Flix Balance: {:?}", admin_flix_balance);

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

#[test]

fn admin_create_channel() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();

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

    // Default reserved usename is "reserved"

    // Try creating a channel with the reserved username
    let res = app
        .execute_contract(
            admin.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::ChannelCreate {
                salt: Binary::default(),
                user_name: "reserved".to_string(),
                description: "reserved".to_string(),
                collaborators: None,
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();

    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(typed_err, &ContractError::UserNameReserved {});

    // Create with admin
    let admin_create_msg = ExecuteMsg::AdminChannelCreate {
        salt: Binary::default(),
        user_name: "reserved".to_string(),
        description: "Description".to_string(),
        collaborators: None,
        recipient: setup_response.test_accounts.creator.clone().into_string(),
    };

    let _res = app
        .execute_contract(
            admin.clone(),
            channel_contract_addr.clone(),
            &admin_create_msg,
            &[],
        )
        .unwrap();
}
