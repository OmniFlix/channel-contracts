use crate::helpers::{
    msg_wrapper::{get_channel_instantiate_msg, CreateChannelMsgBuilder},
    setup::setup,
    utils::get_event_attribute,
};
use channel_manager::error::ChannelError;
use cosmwasm_std::coin;
use cw_multi_test::Executor;
use omniflix_channel::ContractError;
use omniflix_channel_types::channel::ChannelDetails;
use omniflix_channel_types::msg::{ExecuteMsg, QueryMsg};

#[test]
fn missing_channel_id() {
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

    // Missing channel_id
    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::ChannelUpdateDetails {
                channel_id: "".to_string(),
                description: Some("creator".to_string()),
                banner_picture: None,
                profile_picture: None,
                channel_name: None,
                payment_address: None,
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
fn invalid_channel() {
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

    // Channel not found
    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::ChannelUpdateDetails {
                channel_id: "1".to_string(),
                description: Some("creator".to_string()),
                banner_picture: None,
                profile_picture: None,
                channel_name: None,
                payment_address: None,
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

    let create_channel_msg = CreateChannelMsgBuilder::new("creator", creator.clone()).build();

    // Create a channel
    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &create_channel_msg,
            &[coin(1000000, "uflix")],
        )
        .unwrap();
    let channel_id = get_event_attribute(res, "wasm", "channel_id");

    // Query Channel Details
    let channel: ChannelDetails = app
        .wrap()
        .query_wasm_smart(
            channel_contract_addr.clone(),
            &QueryMsg::ChannelDetails {
                channel_id: channel_id.clone(),
            },
        )
        .unwrap();
    assert_eq!(channel.channel_id, channel_id);

    // Unauthorized
    let res = app
        .execute_contract(
            collector.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::ChannelUpdateDetails {
                channel_id: channel_id.clone(),
                description: Some("creator".to_string()),
                banner_picture: None,
                profile_picture: None,
                channel_name: None,
                payment_address: None,
            },
            &[],
        )
        .unwrap_err();

    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(typed_err, &ContractError::Unauthorized {});
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

    let create_channel_msg = CreateChannelMsgBuilder::new("creator", creator.clone()).build();
    // Create a channel
    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &create_channel_msg,
            &[coin(1000000, "uflix")],
        )
        .unwrap();
    let channel_id = get_event_attribute(res, "wasm", "channel_id");

    // Query Channel
    let channel: ChannelDetails = app
        .wrap()
        .query_wasm_smart(
            channel_contract_addr.clone(),
            &QueryMsg::ChannelDetails {
                channel_id: channel_id.clone(),
            },
        )
        .unwrap();
    assert_eq!(channel.channel_id, channel_id);

    // Happy path
    let _res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::ChannelUpdateDetails {
                channel_id: channel_id.clone(),
                description: Some("new description".to_string()),
                banner_picture: None,
                profile_picture: None,
                channel_name: None,
                payment_address: Some(admin.clone().to_string()),
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap();

    // Query Channel Details
    let channel: ChannelDetails = app
        .wrap()
        .query_wasm_smart(
            channel_contract_addr.clone(),
            &QueryMsg::ChannelDetails {
                channel_id: channel_id.clone(),
            },
        )
        .unwrap();
    assert_eq!(channel.payment_address, admin.clone());
}

#[test]
fn invalid() {
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

    // Create a channel
    let create_channel_msg = CreateChannelMsgBuilder::new("creator", creator.clone()).build();
    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &create_channel_msg,
            &[coin(1000000, "uflix")],
        )
        .unwrap();
    let channel_id = get_event_attribute(res, "wasm", "channel_id");

    // Query Channel Details
    let channel: ChannelDetails = app
        .wrap()
        .query_wasm_smart(
            channel_contract_addr.clone(),
            &QueryMsg::ChannelDetails {
                channel_id: channel_id.clone(),
            },
        )
        .unwrap();
    assert_eq!(channel.channel_id, channel_id);

    // Invalid banner link
    let _res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::ChannelUpdateDetails {
                channel_id: channel_id.clone(),
                description: None,
                banner_picture: Some("i".repeat(1001)),
                profile_picture: None,
                channel_name: None,
                payment_address: None,
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();

    // Invalid profile link
    let _res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::ChannelUpdateDetails {
                channel_id: channel_id.clone(),
                description: None,
                banner_picture: None,
                profile_picture: Some("i".repeat(1001)),
                channel_name: None,
                payment_address: None,
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();

    // Invalid channel name
    let _res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::ChannelUpdateDetails {
                channel_id: channel_id.clone(),
                description: None,
                banner_picture: None,
                profile_picture: None,
                // No special characters
                channel_name: Some("creator_1".to_string()),
                payment_address: None,
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();
}
