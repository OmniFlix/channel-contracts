use asset_manager::types::Playlist;
use channel_types::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cosmwasm_std::{coin, Binary};
use cw_multi_test::Executor;
use omniflix_channel::ContractError;

use crate::helpers::setup::setup;
use crate::helpers::utils::{create_denom_msg, get_event_attribute, mint_onft_msg};

#[test]
fn empty_playlist() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();
    let creator = setup_response.test_accounts.creator.clone();
    let _collector = setup_response.test_accounts.collector.clone();

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

    // Query the playlist
    let query_msg = QueryMsg::Playlist {
        channel_id: channel_id.clone(),
        playlist_name: "My Playlist".to_string(),
    };

    let playlist: Playlist = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();

    assert_eq!(playlist.assets.len(), 0);

    // Refresh the playlist
    let refresh_playlist_msg = ExecuteMsg::PlaylistRefresh {
        channel_id: channel_id.clone(),
        playlist_name: "My Playlist".to_string(),
    };

    let _res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &refresh_playlist_msg,
            &[],
        )
        .unwrap();

    // Query the playlist
    let playlist: Playlist = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();

    assert_eq!(playlist.assets.len(), 0);
}

#[test]
fn playlist_with_assets() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();
    let creator = setup_response.test_accounts.creator.clone();
    let _collector = setup_response.test_accounts.collector.clone();

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

    // Create an asset
    let asset_collection_id = "id".to_string();
    let asset_id = "asset_id".to_string();

    let create_denom_msg = create_denom_msg(
        creator.clone().to_string(),
        asset_collection_id.clone(),
        Some("Media asset collection".to_string()),
    );
    let _res = app.execute(creator.clone(), create_denom_msg);
    let mint_onft_asset1_msg = mint_onft_msg(
        "id".to_string(),
        "asset_id".to_string(),
        creator.clone().to_string(),
    );
    let _res = app.execute(creator.clone(), mint_onft_asset1_msg);

    // Mint another asset

    let mint_onft_asset2_msg = mint_onft_msg(
        "id".to_string(),
        "asset_id2".to_string(),
        creator.clone().to_string(),
    );
    let _res = app.execute(creator.clone(), mint_onft_asset2_msg);

    // Publish the assets
    let publish_msg = ExecuteMsg::Publish {
        asset_type: asset_manager::types::AssetType::Nft {
            collection_id: asset_collection_id.clone(),
            onft_id: asset_id.clone(),
        },
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

    let publish_msg = ExecuteMsg::Publish {
        asset_type: asset_manager::types::AssetType::Nft {
            collection_id: asset_collection_id.clone(),
            onft_id: "asset_id2".to_string(),
        },
        salt: Binary::from("salt2".as_bytes()),
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
    let publish_id2 = get_event_attribute(res.clone(), "wasm", "publish_id");

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

    // Add an asset to the playlist
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

    let add_asset_msg = ExecuteMsg::PlaylistAddAsset {
        publish_id: publish_id2.clone(),
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

    // Query the playlist
    let query_msg = QueryMsg::Playlist {
        channel_id: channel_id.clone(),
        playlist_name: "My Playlist".to_string(),
    };

    let playlist: Playlist = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();

    assert_eq!(playlist.assets.len(), 2);
    assert_eq!(
        playlist.assets[0].asset_type,
        asset_manager::types::AssetType::Nft {
            collection_id: asset_collection_id.clone(),
            onft_id: asset_id.clone(),
        }
    );
    assert_eq!(
        playlist.assets[1].asset_type,
        asset_manager::types::AssetType::Nft {
            collection_id: asset_collection_id.clone(),
            onft_id: "asset_id2".to_string(),
        }
    );

    // Refresh the playlist
    let refresh_playlist_msg = ExecuteMsg::PlaylistRefresh {
        channel_id: channel_id.clone(),
        playlist_name: "My Playlist".to_string(),
    };

    let _res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &refresh_playlist_msg,
            &[],
        )
        .unwrap();

    // Query the playlist
    let playlist: Playlist = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();

    // Playlist should have the same assets
    assert_eq!(playlist.assets.len(), 2);
    assert_eq!(
        playlist.assets[0].asset_type,
        asset_manager::types::AssetType::Nft {
            collection_id: asset_collection_id.clone(),
            onft_id: asset_id.clone(),
        }
    );
    assert_eq!(
        playlist.assets[1].asset_type,
        asset_manager::types::AssetType::Nft {
            collection_id: asset_collection_id.clone(),
            onft_id: "asset_id2".to_string(),
        }
    );
}

#[test]
fn playlist_with_assets_and_removed_assets() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();
    let creator = setup_response.test_accounts.creator.clone();
    let _collector = setup_response.test_accounts.collector.clone();

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

    // Create an asset
    let asset_collection_id = "id".to_string();
    let asset_id = "asset_id".to_string();

    let create_denom_msg = create_denom_msg(
        creator.clone().to_string(),
        asset_collection_id.clone(),
        Some("Media asset collection".to_string()),
    );
    let _res = app.execute(creator.clone(), create_denom_msg);
    let mint_onft_asset1_msg = mint_onft_msg(
        "id".to_string(),
        "asset_id".to_string(),
        creator.clone().to_string(),
    );
    let _res = app.execute(creator.clone(), mint_onft_asset1_msg);

    // Mint another asset

    let mint_onft_asset2_msg = mint_onft_msg(
        "id".to_string(),
        "asset_id2".to_string(),
        creator.clone().to_string(),
    );

    let _res = app.execute(creator.clone(), mint_onft_asset2_msg);

    // Publish the assets
    let publish_msg = ExecuteMsg::Publish {
        asset_type: asset_manager::types::AssetType::Nft {
            collection_id: asset_collection_id.clone(),
            onft_id: asset_id.clone(),
        },
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

    let publish_msg = ExecuteMsg::Publish {
        asset_type: asset_manager::types::AssetType::Nft {
            collection_id: asset_collection_id.clone(),
            onft_id: "asset_id2".to_string(),
        },
        salt: Binary::from("salt2".as_bytes()),
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

    let publish_id2 = get_event_attribute(res.clone(), "wasm", "publish_id");

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

    // Add an asset to the playlist

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

    let add_asset_msg = ExecuteMsg::PlaylistAddAsset {
        publish_id: publish_id2.clone(),
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

    // Query the playlist
    let query_msg = QueryMsg::Playlist {
        channel_id: channel_id.clone(),
        playlist_name: "My Playlist".to_string(),
    };

    let playlist: Playlist = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();

    assert_eq!(playlist.assets.len(), 2);
    assert_eq!(
        playlist.assets[0].asset_type,
        asset_manager::types::AssetType::Nft {
            collection_id: asset_collection_id.clone(),
            onft_id: asset_id.clone(),
        }
    );
    assert_eq!(
        playlist.assets[1].asset_type,
        asset_manager::types::AssetType::Nft {
            collection_id: asset_collection_id.clone(),
            onft_id: "asset_id2".to_string(),
        }
    );

    // Unpublish the first asset
    let unpublish_msg = ExecuteMsg::Unpublish {
        publish_id: publish_id.clone(),
        channel_id: channel_id.clone(),
    };

    let _res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &unpublish_msg,
            &[],
        )
        .unwrap();

    // Refresh the playlist
    let refresh_playlist_msg = ExecuteMsg::PlaylistRefresh {
        channel_id: channel_id.clone(),
        playlist_name: "My Playlist".to_string(),
    };

    let _res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &refresh_playlist_msg,
            &[],
        )
        .unwrap();

    // Query the playlist
    let playlist: Playlist = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();

    // Playlist should have the second asset only
    assert_eq!(playlist.assets.len(), 1);
    assert_eq!(
        playlist.assets[0].asset_type,
        asset_manager::types::AssetType::Nft {
            collection_id: asset_collection_id.clone(),
            onft_id: "asset_id2".to_string(),
        }
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
    let creator2 = setup_response.test_accounts.creator2.clone();

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

    // Try refreshing the playlist with a different user
    let refresh_playlist_msg = ExecuteMsg::PlaylistRefresh {
        channel_id: channel_id.clone(),
        playlist_name: "My Playlist".to_string(),
    };

    let res = app
        .execute_contract(
            creator2.clone(),
            channel_contract_addr.clone(),
            &refresh_playlist_msg,
            &[],
        )
        .unwrap_err();

    let error = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        error,
        &ContractError::OnftNotOwned {
            collection_id: "Channels".to_string(),
            onft_id: channel_id.clone().replace("channel", "onft")
        }
    );
}
