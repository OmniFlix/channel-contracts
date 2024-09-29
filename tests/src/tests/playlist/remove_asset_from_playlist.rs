use asset_manager::error::PlaylistError;
use asset_manager::types::Playlist;
use channel_types::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cosmwasm_std::{coin, Binary};
use cw_multi_test::Executor;
use omniflix_channel::ContractError;

use crate::helpers::setup::setup;
use crate::helpers::utils::{create_denom_msg, get_event_attribute, mint_onft_msg};

#[test]
fn asset_not_in_playlist() {
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
    let create_channel_msg = ExecuteMsg::CreateChannel {
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
        playlist_name: "My Playlist".to_string(),
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
    // Validate the creator have added a playlist

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
    assert_eq!(playlists[0].playlist_name, "My Playlist");

    // Remove an unexisting asset from a playlist
    let remove_asset_msg = ExecuteMsg::PlaylistRemoveAsset {
        publish_id: "publish_id".to_string(),
        channel_id: channel_id.clone(),
        playlist_name: "My Playlist".to_string(),
    };

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &remove_asset_msg,
            &[],
        )
        .unwrap_err();

    let err = res.source().unwrap();
    let typed_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::Playlist(PlaylistError::AssetNotInPlaylist {})
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
    let create_channel_msg = ExecuteMsg::CreateChannel {
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

    // Remove an asset from a playlist that does not exist
    let remove_asset_msg = ExecuteMsg::PlaylistRemoveAsset {
        publish_id: "publish_id".to_string(),
        channel_id: channel_id.clone(),
        playlist_name: "My Playlist".to_string(),
    };

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &remove_asset_msg,
            &[],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let typed_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::Playlist(PlaylistError::PlaylistNotFound {})
    );

    // Create a playlist
    let create_playlist_msg = ExecuteMsg::PlaylistCreate {
        playlist_name: "My Playlist".to_string(),
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

    // Validate the creator have added a playlist
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
    assert_eq!(playlists[0].playlist_name, "My Playlist");
    assert_eq!(playlists[0].assets.len(), 0);

    // Playlist does not have any assets
    // Remove an asset from a playlist

    let remove_asset_msg = ExecuteMsg::PlaylistRemoveAsset {
        publish_id: "publish_id".to_string(),
        channel_id: channel_id.clone(),
        playlist_name: "My Playlist".to_string(),
    };

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &remove_asset_msg,
            &[],
        )
        .unwrap_err();

    let err = res.source().unwrap();
    let typed_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::Playlist(PlaylistError::AssetNotInPlaylist {})
    );
}

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
    let create_channel_msg = ExecuteMsg::CreateChannel {
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
        playlist_name: "My Playlist".to_string(),
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
    // Validate the creator have added a playlist
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
    assert_eq!(playlists[0].playlist_name, "My Playlist");

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
        creator.clone().to_string(),
    );
    let _res = app.execute(creator.clone(), mint_onft_msg);

    let publish_msg = ExecuteMsg::Publish {
        asset_onft_collection_id: asset_collection_id.clone(),
        asset_onft_id: asset_id.clone(),
        salt: Binary::from("salt".as_bytes()),
        channel_id: channel_id.clone(),
        playlist_name: Some("My Playlist".to_string()),
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
    let publish_id = get_event_attribute(res, "wasm", "publish_id");

    // Add an asset to a playlist
    let add_asset_msg = ExecuteMsg::PlaylistAddAsset {
        publish_id: publish_id.clone(),
        asset_channel_id: channel_id.clone(),
        channel_id: channel_id.clone(),
        playlist_name: "My Playlist".to_string(),
    };

    let _res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &add_asset_msg,
            &[],
        )
        .unwrap();

    // Validate the asset was added to the playlist
    let query_msg = QueryMsg::Playlist {
        channel_id: channel_id.clone(),
        playlist_name: "My Playlist".to_string(),
    };

    let playlist: Playlist = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();

    assert_eq!(playlist.assets.len(), 1);

    // Remove an asset from a playlist
    let remove_asset_msg = ExecuteMsg::PlaylistRemoveAsset {
        publish_id: publish_id.clone(),
        channel_id: channel_id.clone(),
        playlist_name: "My Playlist".to_string(),
    };

    let res = app
        .execute_contract(
            collector.clone(),
            channel_contract_addr.clone(),
            &remove_asset_msg,
            &[],
        )
        .unwrap_err();

    let err = res.source().unwrap();
    let typed_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::OnftNotOwned {
            collection_id: "Channels".to_string(),
            onft_id: channel_id.clone().replace("channel", "onft")
        }
    );
}
