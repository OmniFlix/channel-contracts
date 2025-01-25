use crate::helpers::msg_wrapper::{get_channel_instantiate_msg, CreateChannelMsgBuilder};
use crate::helpers::setup::setup;
use crate::helpers::utils::get_event_attribute;
use cosmwasm_std::{coin, Addr, Decimal, Uint128};
use cw_multi_test::Executor;
use omniflix_channel::ContractError;
use omniflix_channel_types::msg::ExecuteMsg;

#[test]
fn invalid_payment_address() {
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
    let channel_create_msg =
        CreateChannelMsgBuilder::new("creator", Addr::unchecked("invalid")).build();

    // Create a channel with an invalid payment address
    let _res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &channel_create_msg.clone(),
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();
}

#[test]
fn invalid_tipping_denom() {
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
    let channel_create_msg = CreateChannelMsgBuilder::new("creator", creator.clone()).build();

    // Create a channel
    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &channel_create_msg.clone(),
            &[coin(1000000, "uflix")],
        )
        .unwrap();

    let channel_id = get_event_attribute(res, "wasm", "channel_id");

    // Channel contracts tipping denom is uflix
    // Try to tip with a different denom
    let res = app
        .execute_contract(
            collector.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::TipCreator {
                channel_id: channel_id.clone(),
                amount: coin(100000, "different_denom"),
            },
            &[coin(1000000, "different_denom")],
        )
        .unwrap_err();

    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(typed_err, &ContractError::InvalidTipDenom {});
}

#[test]
fn invalid_tip_amount() {
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
    let channel_create_msg = CreateChannelMsgBuilder::new("creator", creator.clone()).build();

    // Create a channel
    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &channel_create_msg.clone(),
            &[coin(1000000, "uflix")],
        )
        .unwrap();

    let channel_id = get_event_attribute(res, "wasm", "channel_id");

    // Amount in the message is different from the amount in the coins
    let res = app
        .execute_contract(
            collector.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::TipCreator {
                channel_id: channel_id.clone(),
                amount: coin(1000, "uflix"),
            },
            &[coin(100, "uflix")],
        )
        .unwrap_err();

    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(typed_err, &ContractError::InvalidTipAmount {});
    // Denom in the message is different from the denom in the coins
    let res = app
        .execute_contract(
            collector.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::TipCreator {
                channel_id: channel_id.clone(),
                amount: coin(1000, "uflix"),
            },
            &[coin(1000, "different_denom")],
        )
        .unwrap_err();

    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::Payment(cw_utils::PaymentError::MissingDenom("uflix".to_string()))
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
    let channel_create_msg = CreateChannelMsgBuilder::new("creator", creator.clone()).build();

    // Create a channel
    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &channel_create_msg.clone(),
            &[coin(1000000, "uflix")],
        )
        .unwrap();

    let channel_id = get_event_attribute(res, "wasm", "channel_id");

    // Tip the creator
    let res = app
        .execute_contract(
            collector.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::TipCreator {
                channel_id: channel_id.clone(),
                amount: coin(100000, "uflix"),
            },
            &[coin(100000, "uflix")],
        )
        .unwrap();

    // Validate the tip was sent to the creator
    let amount = get_event_attribute(res.clone(), "transfer", "amount");
    assert_eq!(amount, "100000uflix");
    let recipient = get_event_attribute(res, "transfer", "recipient");
    assert_eq!(recipient, creator.to_string());

    // Add collaborator
    let msg = ExecuteMsg::ChannelAddCollaborator {
        channel_id: channel_id.clone(),
        collaborator_address: collector.clone().into_string(),
        collaborator_details: omniflix_channel_types::channel::ChannelCollaborator {
            role: omniflix_channel_types::channel::Role::Moderator,
            share: Decimal::from_ratio(Uint128::one(), Uint128::from(3u128)),
        },
    };

    let _res = app
        .execute_contract(creator.clone(), channel_contract_addr.clone(), &msg, &[])
        .unwrap();

    // Execute a tip
    let res = app
        .execute_contract(
            collector.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::TipCreator {
                channel_id: channel_id.clone(),
                amount: coin(100000, "uflix"),
            },
            &[coin(100000, "uflix")],
        )
        .unwrap();

    // Validate the tip was sent to the creator
    let amount = get_event_attribute(res.clone(), "wasm", &creator.into_string());
    assert_eq!(amount, "66667uflix");

    let amount = get_event_attribute(res.clone(), "wasm", &collector.into_string());
    assert_eq!(amount, "33333uflix");
}
