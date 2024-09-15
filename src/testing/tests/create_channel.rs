use crate::channels::ChannelDetails;
use crate::msg::{ExecuteMsg, QueryMsg};
use crate::state::ChannelConractConfig;
use crate::ContractError;
use crate::{msg::InstantiateMsg, testing::setup::setup};
use cosmwasm_std::{coin, Binary};
use cw_multi_test::Executor;

#[test]
fn create_channel() {
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
            &ExecuteMsg::CreateChannel {
                salt: Binary::default(),
                user_name: "creator".to_string(),
                description: "creator".to_string(),
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

    // Send more than the required fee
    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::CreateChannel {
                salt: Binary::default(),
                user_name: "creator".to_string(),
                description: "creator".to_string(),
            },
            &[coin(1000001, "uflix")],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let typed_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::PaymentError {
            expected: [coin(1000000, "uflix")].to_vec(),
            received: [coin(1000001, "uflix")].to_vec()
        }
    );

    // Too long username
    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::CreateChannel {
                salt: Binary::default(),
                user_name: "creatorcreatorcreatorcreatorcreator".to_string(),
                description: "creator".to_string(),
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let typed_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(typed_err, &ContractError::InvalidUserName {});

    // Too long description
    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::CreateChannel {
                salt: Binary::default(),
                user_name: "creator".to_string(),
                // Generate a sting with 257 characters
                description: "a".repeat(257),
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let typed_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(typed_err, &ContractError::InvalidDescription {});

    // Happy path
    let _res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::CreateChannel {
                salt: Binary::default(),
                user_name: "creator".to_string(),
                description: "creator".to_string(),
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap();

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
}
