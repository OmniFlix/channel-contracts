use asset_manager::{
    error::PlaylistError,
    types::{Asset, Playlist},
};
use channel_types::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cosmwasm_std::{coin, Binary, CosmosMsg};
use cw_multi_test::Executor;
use omniflix_channel::ContractError;

use crate::helpers::{
    setup::setup,
    utils::{create_denom_msg, get_event_attribute, mint_onft_msg},
};

#[test]
fn asset_does_not_exist() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();
    let creator = setup_response.test_accounts.creator.clone();

    // Instantiate Channel Contract
    let instantiate_msg = InstantiateMsg {
        admin: setup_response.test_accounts.admin.clone(),
        channel_creation_fee: vec![],
        fee_collector: setup_response.test_accounts.admin,
        channels_collection_id: "Channels".to_string(),
        channels_collection_name: "Channels".to_string(),
        channels_collection_symbol: "CH".to_string(),
    };

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
    let create_channel_msg = ExecuteMsg::ChannelCreate {
        salt: Binary::from("salt".as_bytes()),
        user_name: "user_name".to_string(),
        description: "description".to_string(),
        collaborators: None,
    };

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &create_channel_msg,
            &[],
        )
        .unwrap();
    // Get the channel_id from the event
    let channel_id = get_event_attribute(res.clone(), "wasm", "channel_id");

    // Try publishing an asset without it existing
    let publish_msg = ExecuteMsg::Publish {
        asset_onft_collection_id: "id".to_string(),
        asset_onft_id: "asset_id".to_string(),
        salt: Binary::from("salt".as_bytes()),
        channel_id: channel_id.clone(),
        playlist_name: None,
        is_visible: true,
    };

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &publish_msg,
            &[],
        )
        .unwrap_err();

    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::OnftNotFound {
            collection_id: "id".to_string(),
            onft_id: "asset_id".to_string()
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

    // Instantiate Channel Contract
    let instantiate_msg = InstantiateMsg {
        admin: setup_response.test_accounts.admin.clone(),
        channel_creation_fee: vec![],
        fee_collector: setup_response.test_accounts.admin,
        channels_collection_id: "Channels".to_string(),
        channels_collection_name: "Channels".to_string(),
        channels_collection_symbol: "CH".to_string(),
    };

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
    let create_channel_msg = ExecuteMsg::ChannelCreate {
        salt: Binary::from("salt".as_bytes()),
        user_name: "user_name".to_string(),
        description: "description".to_string(),
        collaborators: None,
    };

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &create_channel_msg,
            &[],
        )
        .unwrap();
    // Get the channel_id from the event
    let channel_id = get_event_attribute(res.clone(), "wasm", "channel_id");

    // We need to create a denom for creator
    // Then we will mint a onft representing the asset
    // Then we will publish the asset

    let asset_collection_id = "id".to_string();
    let asset_id = "asset_id".to_string();

    let create_denom_msg = create_denom_msg(
        creator.clone().to_string(),
        asset_collection_id.clone(),
        Some("Media asset collection".to_string()),
    );
    let _res = app.execute(creator.clone(), create_denom_msg);
    let mint_onft_msg = mint_onft_msg(
        asset_collection_id.clone(),
        asset_id.clone(),
        creator.clone().to_string(),
    );
    let cosmos_msg: CosmosMsg = mint_onft_msg.into();
    let _res = app.execute(creator.clone(), cosmos_msg);

    // Publish the asset
    let publish_msg = ExecuteMsg::Publish {
        asset_onft_collection_id: asset_collection_id.clone(),
        asset_onft_id: asset_id.clone(),
        salt: Binary::from("salt".as_bytes()),
        channel_id: channel_id.clone(),
        playlist_name: None,
        is_visible: true,
    };

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &publish_msg,
            &[],
        )
        .unwrap();

    let publish_id = get_event_attribute(res.clone(), "wasm", "publish_id");

    // Query the asset
    let query_msg = QueryMsg::Asset {
        channel_id: channel_id.clone(),
        publish_id: publish_id.clone(),
    };

    let asset: Asset = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();

    assert_eq!(asset.publish_id, publish_id);
    assert_eq!(asset.collection_id, asset_collection_id);
    assert_eq!(asset.onft_id, asset_id);
}

#[test]
fn channel_not_owned() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();
    let creator = setup_response.test_accounts.creator.clone();
    let collector = setup_response.test_accounts.collector.clone();

    // Instantiate Channel Contract
    let instantiate_msg = InstantiateMsg {
        admin: setup_response.test_accounts.admin.clone(),
        channel_creation_fee: vec![],
        fee_collector: setup_response.test_accounts.admin,
        channels_collection_id: "Channels".to_string(),
        channels_collection_name: "Channels".to_string(),
        channels_collection_symbol: "CH".to_string(),
    };

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
    let create_channel_msg = ExecuteMsg::ChannelCreate {
        salt: Binary::from("salt".as_bytes()),
        user_name: "user_name".to_string(),
        description: "description".to_string(),
        collaborators: None,
    };

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &create_channel_msg,
            &[],
        )
        .unwrap();
    // Get the channel_id from the event
    let channel_id = get_event_attribute(res.clone(), "wasm", "channel_id");
    let channel_onft_id = get_event_attribute(res.clone(), "wasm", "onft_id");
    // Creator owns the channel
    // Lets create a asset for collector and try to publish it

    let asset_collection_id = "id".to_string();
    let asset_id = "asset_id".to_string();

    let create_denom_msg = create_denom_msg(
        collector.clone().to_string(),
        asset_collection_id.clone(),
        Some("Media asset collection".to_string()),
    );
    let _res = app.execute(creator.clone(), create_denom_msg);
    let mint_onft_msg = mint_onft_msg(
        "id".to_string(),
        "asset_id".to_string(),
        collector.clone().to_string(),
    );
    let cosmos_msg: CosmosMsg = mint_onft_msg.into();
    let _res = app.execute(collector.clone(), cosmos_msg);

    // Publish the asset
    let publish_msg = ExecuteMsg::Publish {
        asset_onft_collection_id: asset_collection_id.clone(),
        asset_onft_id: asset_id.clone(),
        salt: Binary::from("salt".as_bytes()),
        channel_id: channel_id.clone(),
        playlist_name: None,
        is_visible: true,
    };

    let res = app
        .execute_contract(
            collector.clone(),
            channel_contract_addr.clone(),
            &publish_msg,
            &[],
        )
        .unwrap_err();

    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::OnftNotOwned {
            collection_id: "Channels".to_string(),
            onft_id: channel_onft_id.clone()
        }
    );
}

#[test]
fn playlist_does_not_exist() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();
    let creator = setup_response.test_accounts.creator.clone();

    // Instantiate Channel Contract
    let instantiate_msg = InstantiateMsg {
        admin: setup_response.test_accounts.admin.clone(),
        channel_creation_fee: vec![],
        fee_collector: setup_response.test_accounts.admin,
        channels_collection_id: "Channels".to_string(),
        channels_collection_name: "Channels".to_string(),
        channels_collection_symbol: "CH".to_string(),
    };

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
    let create_channel_msg = ExecuteMsg::ChannelCreate {
        salt: Binary::from("salt".as_bytes()),
        user_name: "user_name".to_string(),
        description: "description".to_string(),
        collaborators: None,
    };

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &create_channel_msg,
            &[],
        )
        .unwrap();
    // Get the channel_id from the event
    let channel_id = get_event_attribute(res.clone(), "wasm", "channel_id");

    // Creator owns the channel
    // Lets create a asset for creator and try to publish it

    let asset_collection_id = "id".to_string();
    let asset_id = "asset_id".to_string();

    let create_denom_msg = create_denom_msg(
        creator.clone().to_string(),
        asset_collection_id.clone(),
        Some("Media asset collection".to_string()),
    );
    let _res = app.execute(creator.clone(), create_denom_msg);
    let mint_onft_msg = mint_onft_msg(
        "id".to_string(),
        "asset_id".to_string(),
        creator.clone().to_string(),
    );
    let cosmos_msg: CosmosMsg = mint_onft_msg.into();
    let _res = app.execute(creator.clone(), cosmos_msg);

    // Publish the asset with wrong playlist name
    let publish_msg = ExecuteMsg::Publish {
        asset_onft_collection_id: asset_collection_id.clone(),
        asset_onft_id: asset_id.clone(),
        salt: Binary::from("salt".as_bytes()),
        channel_id: channel_id.clone(),
        playlist_name: Some("Wrong playlist".to_string()),
        is_visible: true,
    };

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &publish_msg,
            &[],
        )
        .unwrap_err();

    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::Playlist(PlaylistError::PlaylistNotFound {})
    );
}

#[test]
fn with_playlist() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();
    let creator = setup_response.test_accounts.creator.clone();

    // Instantiate Channel Contract
    let instantiate_msg = InstantiateMsg {
        admin: setup_response.test_accounts.admin.clone(),
        channel_creation_fee: vec![],
        fee_collector: setup_response.test_accounts.admin,
        channels_collection_id: "Channels".to_string(),
        channels_collection_name: "Channels".to_string(),
        channels_collection_symbol: "CH".to_string(),
    };

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
    let create_channel_msg = ExecuteMsg::ChannelCreate {
        salt: Binary::from("salt".as_bytes()),
        user_name: "user_name".to_string(),
        description: "description".to_string(),
        collaborators: None,
    };

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &create_channel_msg,
            &[],
        )
        .unwrap();
    // Get the channel_id from the event
    let channel_id = get_event_attribute(res.clone(), "wasm", "channel_id");

    // Creator owns the channel
    // Lets create a asset for creator and try to publish it

    let asset_collection_id = "id".to_string();
    let asset_id = "asset_id".to_string();

    let create_denom_msg = create_denom_msg(
        creator.clone().to_string(),
        asset_collection_id.clone(),
        Some("Media asset collection".to_string()),
    );
    let _res = app.execute(creator.clone(), create_denom_msg);
    let mint_onft_msg = mint_onft_msg(
        "id".to_string(),
        "asset_id".to_string(),
        creator.clone().to_string(),
    );
    let cosmos_msg: CosmosMsg = mint_onft_msg.into();
    let _res = app.execute(creator.clone(), cosmos_msg);

    // Create a playlist
    let create_playlist_msg = ExecuteMsg::PlaylistCreate {
        playlist_name: "My Videos".to_string(),
        channel_id: channel_id.clone(),
    };

    let _res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &create_playlist_msg,
            &[],
        )
        .unwrap();

    // Publish the asset under the new playlist
    let publish_msg = ExecuteMsg::Publish {
        asset_onft_collection_id: asset_collection_id.clone(),
        asset_onft_id: asset_id.clone(),
        salt: Binary::from("salt".as_bytes()),
        channel_id: channel_id.clone(),
        playlist_name: Some("My Videos".to_string()),
        is_visible: true,
    };

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &publish_msg,
            &[],
        )
        .unwrap();

    let publish_id = get_event_attribute(res.clone(), "wasm", "publish_id");

    // Query the new playlist
    let query_msg = QueryMsg::Playlist {
        channel_id: channel_id.clone(),
        playlist_name: "My Videos".to_string(),
    };

    let playlist: Playlist = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();

    assert_eq!(playlist.assets.len(), 1);
    assert_eq!(playlist.assets[0].publish_id, publish_id);
}

#[test]
fn asset_not_owned() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();
    let creator = setup_response.test_accounts.creator.clone();
    let collector = setup_response.test_accounts.collector.clone();

    // Instantiate Channel Contract
    let instantiate_msg = InstantiateMsg {
        admin: setup_response.test_accounts.admin.clone(),
        channel_creation_fee: vec![],
        fee_collector: setup_response.test_accounts.admin,
        channels_collection_id: "Channels".to_string(),
        channels_collection_name: "Channels".to_string(),
        channels_collection_symbol: "CH".to_string(),
    };

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
    let create_channel_msg = ExecuteMsg::ChannelCreate {
        salt: Binary::from("salt".as_bytes()),
        user_name: "user_name".to_string(),
        description: "description".to_string(),
        collaborators: None,
    };

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &create_channel_msg,
            &[],
        )
        .unwrap();

    // Get the channel_id from the event
    let channel_id = get_event_attribute(res.clone(), "wasm", "channel_id");

    // Publish an asset
    let asset_collection_id = "id".to_string();
    let asset_id = "asset_id".to_string();

    let create_denom_msg = create_denom_msg(
        creator.clone().to_string(),
        asset_collection_id.clone(),
        Some("Media asset collection".to_string()),
    );
    let _res = app.execute(creator.clone(), create_denom_msg);
    let mint_onft_msg = mint_onft_msg(
        "id".to_string(),
        "asset_id".to_string(),
        collector.clone().to_string(),
    );
    let _res = app.execute(creator.clone(), mint_onft_msg);

    // Asset is owned by collector
    // Creator tries to publish the asset
    let publish_msg = ExecuteMsg::Publish {
        asset_onft_collection_id: asset_collection_id.clone(),
        asset_onft_id: asset_id.clone(),
        salt: Binary::from("salt".as_bytes()),
        channel_id: channel_id.clone(),
        playlist_name: None,
        is_visible: true,
    };

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &publish_msg,
            &[],
        )
        .unwrap_err();

    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::OnftNotOwned {
            collection_id: asset_collection_id.clone(),
            onft_id: asset_id.clone()
        }
    );
}
