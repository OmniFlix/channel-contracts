use asset_manager::types::Playlist;
use channel_manager::types::ChannelDetails;
use channel_types::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{coin, Binary, BlockInfo, Timestamp};
use cw_multi_test::Executor;
use omniflix_channel::helpers::generate_random_id_with_prefix;
use omniflix_channel::ContractError;

use crate::helpers::setup::setup;
use crate::helpers::utils::get_event_attribute;

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

    // Send more than the required fee
    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::CreateChannel {
                salt: Binary::default(),
                user_name: "creator".to_string(),
                description: "creator".to_string(),
                collabarators: None,
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
                collabarators: None,
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
                collabarators: None,
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let typed_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(typed_err, &ContractError::InvalidDescription {});

    // Happy path
    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::CreateChannel {
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
    assert_eq!(recipient, admin);

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

    // Validate the playlist created by the channel
    let channel_id = channels[0].channel_id.clone();

    let playlists: Vec<Playlist> = app
        .wrap()
        .query_wasm_smart(
            channel_contract_addr.clone(),
            &QueryMsg::Playlists {
                channel_id: channel_id.clone(),
                start_after: None,
                limit: None,
            },
        )
        .unwrap();
    assert_eq!(playlists.len(), 1);
    // Default playlist name is "My Videos"
    assert_eq!(playlists[0].playlist_name, "My Videos");
    assert_eq!(playlists[0].assets.len(), 0);
}

#[test]
fn set_channel_details() {
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

    // Create a channel
    let _res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::CreateChannel {
                salt: Binary::default(),
                user_name: "creator".to_string(),
                description: "creator".to_string(),
                collabarators: None,
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
    // Current channel_id
    // Its random, so we need to get it from the query
    let channel_id = channel.channel_id.clone();
    let onft_id = channel.onft_id.clone();

    // Missing channel_id
    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::SetChannelDetails {
                channel_id: "".to_string(),
                description: "creator".to_string(),
            },
            &[],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let typed_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(typed_err, &ContractError::ChannelIdNotFound {});

    // Missing description
    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::SetChannelDetails {
                channel_id: channel_id.clone(),
                description: "".to_string(),
            },
            &[],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let typed_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(typed_err, &ContractError::InvalidDescription {});

    // Channel not found
    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::SetChannelDetails {
                channel_id: "1".to_string(),
                description: "creator".to_string(),
            },
            &[],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let typed_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(typed_err, &ContractError::ChannelIdNotFound {});

    // Unauthorized
    let res = app
        .execute_contract(
            collector.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::SetChannelDetails {
                channel_id: channel_id.clone(),
                description: "creator".to_string(),
            },
            &[],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let typed_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::OnftNotOwned {
            collection_id: "Channels".to_string(),
            onft_id: onft_id.clone()
        }
    );

    // Happy path
    let _res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::SetChannelDetails {
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

#[test]
fn same_user_name() {
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

    // Create a channel
    let _res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::CreateChannel {
                salt: Binary::default(),
                user_name: "creator".to_string(),
                description: "creator".to_string(),
                collabarators: None,
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap();

    // Collector tries to create a channel with the same user_name
    // We are using diffirent salt to avoid the channel_id collision
    // We can use the same salt, but change the env variables as if the transaction is in a different block
    let res = app
        .execute_contract(
            collector.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::CreateChannel {
                salt: Binary::from("salt".as_bytes()),
                user_name: "creator".to_string(),
                description: "creator".to_string(),
                collabarators: None,
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let typed_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(typed_err, &ContractError::UserNameAlreadyTaken {});

    // Set block params to simulate a different block
    app.set_block(BlockInfo {
        height: 123123,
        time: Timestamp::from_nanos(123123),
        chain_id: "test".to_string(),
    });

    // Collector tries to create a channel with the same user_name
    // This time we are using the same salt
    let res = app
        .execute_contract(
            collector.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::CreateChannel {
                salt: Binary::default(),
                user_name: "creator".to_string(),
                description: "creator".to_string(),
                collabarators: None,
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let typed_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(typed_err, &ContractError::UserNameAlreadyTaken {});
}

#[test]
fn random_id_generator() {
    let channel_id = generate_random_id_with_prefix(
        &Binary::from_base64("salt").unwrap(),
        &mock_env(),
        "channel",
    );

    let onft_id =
        generate_random_id_with_prefix(&Binary::from_base64("salt").unwrap(), &mock_env(), "onft");

    // remove the prefixes
    let channel_id = channel_id.split_at(7).1;
    let onft_id = onft_id.split_at(4).1;

    // they should be the same
    assert_eq!(channel_id, onft_id);
}
