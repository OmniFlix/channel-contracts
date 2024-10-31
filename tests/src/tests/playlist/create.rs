use asset_manager::error::PlaylistError;
use asset_manager::types::Playlist;
use channel_types::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cosmwasm_std::{coin, Binary};
use cw_multi_test::Executor;
use omniflix_channel::ContractError;

use crate::helpers::setup::setup;
use crate::helpers::utils::get_event_attribute;

// Create a playlist that already exists
#[test]
fn already_exists() {
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
        collabarators: None,
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

    // Verify that the playlist exists under the channel
    let query_msg = QueryMsg::Playlists {
        channel_id: channel_id.clone(),
        limit: None,
        start_after: None,
    };

    let playlists: Vec<Playlist> = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();

    assert_eq!(playlists.len(), 1);
    assert_eq!(playlists[0].playlist_name, "My Videos");

    // Create a playlist that already exists
    let create_playlist_msg = ExecuteMsg::PlaylistCreate {
        playlist_name: "My Videos".to_string(),
        channel_id: channel_id.clone(),
    };

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &create_playlist_msg,
            &[],
        )
        .unwrap_err();

    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::Playlist(PlaylistError::PlaylistAlreadyExists {})
    );
}

// Create a playlist without owning the channel
#[test]
fn not_owned() {
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
        collabarators: None,
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

    // Create a playlist without owning the channel
    let create_playlist_msg = ExecuteMsg::PlaylistCreate {
        playlist_name: "My Playlist".to_string(),
        channel_id: channel_id.clone(),
    };

    let res = app
        .execute_contract(
            // Collector does not own the channel
            collector.clone(),
            channel_contract_addr.clone(),
            &create_playlist_msg,
            &[],
        )
        .unwrap_err();

    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::OnftNotOwned {
            onft_id: channel_onft_id.clone(),
            collection_id: "Channels".to_string(),
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
        collabarators: None,
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

    // Verify that the playlist exists under the channel
    let query_msg = QueryMsg::Playlists {
        channel_id: channel_id.clone(),
        limit: None,
        start_after: None,
    };

    let playlists: Vec<Playlist> = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();

    assert_eq!(playlists.len(), 1);
    assert_eq!(playlists[0].playlist_name, "My Videos");
}

#[test]
fn try_creating_same_playlist() {
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
        collabarators: None,
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

    // Verify that the playlist exists under the channel
    let query_msg = QueryMsg::Playlists {
        channel_id: channel_id.clone(),
        limit: None,
        start_after: None,
    };

    let playlists: Vec<Playlist> = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();

    assert_eq!(playlists.len(), 1);
    assert_eq!(playlists[0].playlist_name, "My Videos");

    // Creator tries to create a playlist named "My Videos"
    let create_playlist_msg = ExecuteMsg::PlaylistCreate {
        playlist_name: "My Videos".to_string(),
        channel_id: channel_id.clone(),
    };

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &create_playlist_msg,
            &[],
        )
        .unwrap_err();


    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::Playlist(PlaylistError::PlaylistAlreadyExists {})
    );
}
