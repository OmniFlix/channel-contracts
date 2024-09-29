use asset_manager::types::Playlist;
use channel_manager::types::ChannelDetails;
use channel_types::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{coin, Binary, BlockInfo, CosmosMsg, Timestamp};
use cw_multi_test::Executor;
use omniflix_channel::helpers::generate_random_id_with_prefix;
use omniflix_channel::ContractError;

use crate::helpers::setup::setup;
use crate::helpers::utils::{create_denom_msg, get_event_attribute, mint_onft_msg};

#[test]
fn create_playlist() {
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
    let channel_onft_id = get_event_attribute(res.clone(), "wasm", "onft_id");

    // Create a playlist that already exists
    let create_playlist_msg = ExecuteMsg::CreatePlaylist {
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
    let err = res.source().unwrap();
    let typed_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(typed_err, &ContractError::PlaylistAlreadyExists {});

    // Create a playlist without owning the channel
    let create_playlist_msg = ExecuteMsg::CreatePlaylist {
        playlist_name: "My Playlist".to_string(),
        channel_id: channel_id.clone(),
    };

    let res = app
        .execute_contract(
            collector.clone(),
            channel_contract_addr.clone(),
            &create_playlist_msg,
            &[],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let typed_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::OnftNotOwned {
            onft_id: channel_onft_id.clone(),
            collection_id: "Channels".to_string(),
        }
    );

    // Create a playlist
    let create_playlist_msg = ExecuteMsg::CreatePlaylist {
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

    // Query the playlists
    let query_msg = QueryMsg::Playlists {
        channel_id: channel_id.clone(),
        limit: None,
        start_after: None,
    };

    let playlists: Vec<Playlist> = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();

    assert_eq!(playlists.len(), 2);
    assert_eq!(playlists[0].playlist_name, "My Playlist");
    assert_eq!(playlists[1].playlist_name, "My Videos");
    assert_eq!(playlists[0].assets.len(), 0);
    assert_eq!(playlists[1].assets.len(), 0);
}

#[test]
fn remove_playlist() {
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
    let channel_onft_id = get_event_attribute(res.clone(), "wasm", "onft_id");

    // Remove a playlist that does not exist
    let remove_playlist_msg = ExecuteMsg::RemovePlaylist {
        playlist_name: "My Playlist".to_string(),
        channel_id: channel_id.clone(),
    };

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &remove_playlist_msg,
            &[],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let typed_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(typed_err, &ContractError::PlaylistNotFound {});

    // Remove a playlist without owning the channel
    let create_playlist_msg = ExecuteMsg::CreatePlaylist {
        playlist_name: "My Playlist".to_string(),
        channel_id: channel_id.clone(),
    };

    let res = app
        .execute_contract(
            collector.clone(),
            channel_contract_addr.clone(),
            &create_playlist_msg,
            &[],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let typed_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::OnftNotOwned {
            onft_id: channel_onft_id.clone(),
            collection_id: "Channels".to_string(),
        }
    );

    // Try to remove the default playlist
    let remove_playlist_msg = ExecuteMsg::RemovePlaylist {
        playlist_name: "My Videos".to_string(),
        channel_id: channel_id.clone(),
    };

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &remove_playlist_msg,
            &[],
        )
        .unwrap_err();

    let err = res.source().unwrap();
    let typed_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(typed_err, &ContractError::CannotDeleteDefaultPlaylist {});

    // Create a playlist
    let create_playlist_msg = ExecuteMsg::CreatePlaylist {
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

    // Add an asset to the playlist

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

    let publish_msg = ExecuteMsg::Publish {
        asset_onft_collection_id: asset_collection_id.clone(),
        asset_onft_id: asset_id.clone(),
        salt: Binary::from("salt".as_bytes()),
        channel_id: channel_id.clone(),
        playlist_name: Some("My Playlist".to_string()),
        is_visible: true,
    };

    let _res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &publish_msg,
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
    assert_eq!(playlist.assets[0].onft_id, asset_id.clone());

    // Remove the playlist
    let remove_playlist_msg = ExecuteMsg::RemovePlaylist {
        playlist_name: "My Playlist".to_string(),
        channel_id: channel_id.clone(),
    };

    let _res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &remove_playlist_msg,
            &[],
        )
        .unwrap();

    // Validate the playlist was removed
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

    // Validate the asset was not removed from the default playlist
    assert_eq!(playlists[0].assets.len(), 1);
    assert_eq!(playlists[0].assets[0].onft_id, asset_id.clone());
}

#[test]
fn try_recreating_same_playlist() {
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

    // Creator tries to create a playlist named "My Videos"
    let create_playlist_msg = ExecuteMsg::CreatePlaylist {
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

    let err = res.source().unwrap();
    let typed_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(typed_err, &ContractError::PlaylistAlreadyExists {});
}

#[test]
fn remove_asset_from_playlist() {
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
    let channel_onft_id = get_event_attribute(res.clone(), "wasm", "onft_id");

    // Remove an asset from a playlist that does not exist
    let remove_asset_msg = ExecuteMsg::RemoveAsset {
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
    assert_eq!(typed_err, &ContractError::PlaylistNotFound {});

    // Remove an unexisting asset from a playlist
    let remove_asset_msg = ExecuteMsg::RemoveAsset {
        publish_id: "publish_id".to_string(),
        channel_id: channel_id.clone(),
        playlist_name: "My Videos".to_string(),
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
    assert_eq!(typed_err, &ContractError::AssetNotInPlaylist {});

    // Create a playlist
    let create_playlist_msg = ExecuteMsg::CreatePlaylist {
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

    // Try to remove an asset from a playlist without owning the channel
    let remove_asset_msg = ExecuteMsg::RemoveAsset {
        publish_id: "publish_id".to_string(),
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
            onft_id: channel_onft_id.clone(),
            collection_id: "Channels".to_string(),
        }
    );

    // Add an asset to the playlist

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
    let publish_id = get_event_attribute(res.clone(), "wasm", "publish_id");

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
    assert_eq!(playlist.assets[0].onft_id, asset_id.clone());

    // Remove the asset from the playlist
    let remove_asset_msg = ExecuteMsg::RemoveAsset {
        publish_id: publish_id.clone(),
        channel_id: channel_id.clone(),
        playlist_name: "My Playlist".to_string(),
    };

    let _res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &remove_asset_msg,
            &[],
        )
        .unwrap();

    // Validate the asset was removed from the playlist
    let playlist: Playlist = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();

    assert_eq!(playlist.assets.len(), 0);

    // Validate the asset was not removed from the default playlist
    let query_msg = QueryMsg::Playlist {
        channel_id: channel_id.clone(),
        playlist_name: "My Videos".to_string(),
    };

    let playlist: Playlist = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();

    assert_eq!(playlist.assets.len(), 1);
    assert_eq!(playlist.assets[0].onft_id, asset_id.clone());
}
