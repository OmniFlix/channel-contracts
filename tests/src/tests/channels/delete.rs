use crate::helpers::msg_wrapper::CreateChannelMsgBuilder;
use crate::helpers::setup::setup;
use crate::helpers::utils::get_event_attribute;
use cosmwasm_std::coin;
use cw_multi_test::Executor;
use omniflix_channel::ContractError;
use omniflix_channel_types::{
    channel::ChannelDetails,
    msg::{ExecuteMsg, QueryMsg},
};

#[test]
fn delete_channel_happy_path() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();
    let creator = setup_response.test_accounts.creator.clone();

    // Instantiate the contract
    let channel_contract_addr = app
        .instantiate_contract(
            setup_response.channel_contract_code_id,
            admin.clone(),
            &crate::helpers::msg_wrapper::get_channel_instantiate_msg(admin.clone()),
            &[coin(1000000, "uflix")],
            "Instantiate Channel Contract",
            None,
        )
        .unwrap();

    // Create a channel first
    let username = "creator";
    let channel_create_msg = CreateChannelMsgBuilder::new(username, creator.clone()).build();

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &channel_create_msg,
            &[],
        )
        .unwrap();

    // Extract channel_id from the response
    let channel_id = get_event_attribute(res.clone(), "wasm", "channel_id");

    // Verify channel exists
    let channel_details_query: ChannelDetails = app
        .wrap()
        .query_wasm_smart(
            channel_contract_addr.clone(),
            &QueryMsg::ChannelDetails {
                channel_id: channel_id.clone(),
            },
        )
        .unwrap();
    assert_eq!(channel_details_query.user_name, username);

    // Delete the channel
    let delete_msg = ExecuteMsg::ChannelDelete {
        channel_id: channel_id.clone(),
    };

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &delete_msg,
            &[],
        )
        .unwrap();

    // Verify correct event attributes
    assert_eq!(
        get_event_attribute(res.clone(), "wasm", "action"),
        "delete_channel"
    );
    assert_eq!(
        get_event_attribute(res.clone(), "wasm", "channel_id"),
        channel_id
    );

    // Verify that the channel no longer exists
    let query_result = app.wrap().query_wasm_smart::<ChannelDetails>(
        channel_contract_addr.clone(),
        &QueryMsg::ChannelDetails {
            channel_id: channel_id.clone(),
        },
    );

    // Query should fail with channel not found error
    assert!(query_result.is_err());
}

#[test]
fn delete_channel_not_creator() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();
    let creator = setup_response.test_accounts.creator.clone();
    let creator2 = setup_response.test_accounts.creator2.clone();

    // Instantiate the contract
    let channel_contract_addr = app
        .instantiate_contract(
            setup_response.channel_contract_code_id,
            admin.clone(),
            &crate::helpers::msg_wrapper::get_channel_instantiate_msg(admin.clone()),
            &[coin(1000000, "uflix")],
            "Instantiate Channel Contract",
            None,
        )
        .unwrap();

    // Create a channel first
    let username = "creator";
    let channel_create_msg = CreateChannelMsgBuilder::new(username, creator.clone()).build();

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &channel_create_msg,
            &[],
        )
        .unwrap();

    // Extract channel_id from the response
    let channel_id = get_event_attribute(res.clone(), "wasm", "channel_id");

    // Try to delete channel with non creator
    let delete_msg = ExecuteMsg::ChannelDelete {
        channel_id: channel_id.clone(),
    };

    let res = app
        .execute_contract(
            creator2.clone(),
            channel_contract_addr.clone(),
            &delete_msg,
            &[],
        )
        .unwrap_err();

    // Verify error is about permissions
    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(typed_err, &ContractError::Unauthorized {});

    // Verify channel still exists
    let channel_details_query: ChannelDetails = app
        .wrap()
        .query_wasm_smart(
            channel_contract_addr.clone(),
            &QueryMsg::ChannelDetails {
                channel_id: channel_id.clone(),
            },
        )
        .unwrap();
    assert_eq!(channel_details_query.user_name, username);
}

#[test]
fn delete_channel_when_paused() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();
    let creator = setup_response.test_accounts.creator.clone();

    // Instantiate the contract
    let channel_contract_addr = app
        .instantiate_contract(
            setup_response.channel_contract_code_id,
            admin.clone(),
            &crate::helpers::msg_wrapper::get_channel_instantiate_msg(admin.clone()),
            &[coin(1000000, "uflix")],
            "Instantiate Channel Contract",
            None,
        )
        .unwrap();

    // Create a channel first
    let username = "creator";
    let channel_create_msg = CreateChannelMsgBuilder::new(username, creator.clone()).build();

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &channel_create_msg,
            &[],
        )
        .unwrap();

    // Extract channel_id from the response
    let channel_id = get_event_attribute(res.clone(), "wasm", "channel_id");

    // Pause the contract
    let _res = app
        .execute_contract(
            admin.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::Pause {},
            &[],
        )
        .unwrap();

    // Try to delete the channel when contract is paused
    let delete_msg = ExecuteMsg::ChannelDelete {
        channel_id: channel_id.clone(),
    };

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &delete_msg,
            &[],
        )
        .unwrap_err();

    // Verify error is about contract being paused
    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::Pause(pauser::PauseError::Paused {})
    );
}

#[test]
fn delete_nonexistent_channel() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();
    let creator = setup_response.test_accounts.creator.clone();

    // Instantiate the contract
    let channel_contract_addr = app
        .instantiate_contract(
            setup_response.channel_contract_code_id,
            admin.clone(),
            &crate::helpers::msg_wrapper::get_channel_instantiate_msg(admin.clone()),
            &[coin(1000000, "uflix")],
            "Instantiate Channel Contract",
            None,
        )
        .unwrap();

    // Try to delete a channel that doesn't exist
    let nonexistent_channel_id = "nonexistent_channel_id";
    let delete_msg = ExecuteMsg::ChannelDelete {
        channel_id: nonexistent_channel_id.to_string(),
    };

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &delete_msg,
            &[],
        )
        .unwrap_err();

    // Verify error is about channel not found
    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::Channel(channel_manager::error::ChannelError::ChannelIdNotFound {})
    );
}
