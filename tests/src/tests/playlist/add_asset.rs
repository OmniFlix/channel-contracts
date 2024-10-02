use asset_manager::types::Playlist;
use channel_types::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cosmwasm_std::{coin, Binary, BlockInfo, Timestamp};
use cw_multi_test::Executor;
use omniflix_channel::ContractError;

use crate::helpers::setup::setup;
use crate::helpers::utils::{create_denom_msg, get_event_attribute, mint_onft_msg};

#[test]
fn asset_not_visible() {
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

    // Publish the asset
    let publish_msg = ExecuteMsg::Publish {
        asset_onft_collection_id: asset_collection_id.clone(),
        asset_onft_id: asset_id.clone(),
        salt: Binary::from("salt".as_bytes()),
        channel_id: channel_id.clone(),
        playlist_name: None,
        is_visible: false,
    };

    let _res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &publish_msg,
            &[],
        )
        .unwrap();

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
        publish_id: asset_id.clone(),
        asset_channel_id: asset_collection_id.clone(),
        channel_id: channel_id.clone(),
        playlist_name: "My Playlist".to_string(),
    };

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &add_asset_msg,
            &[],
        )
        .unwrap_err();

    let error = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        error,
        &ContractError::Asset(asset_manager::error::AssetError::AssetNotFound {})
    );
}
#[test]
fn asset_from_diffirent_channel() {
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

    // Creator 1 creates a channel
    let create_channel_msg = ExecuteMsg::ChannelCreate {
        salt: Binary::from("salt".as_bytes()),
        user_name: "Creator 1".to_string(),
        description: "Creator 1 Description".to_string(),
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
    let creator1_channel_id = get_event_attribute(res.clone(), "wasm", "channel_id");

    // Creator 2 creates a channel
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 5_000_000,
        time: Timestamp::from_nanos(5_000_000),
    });
    let create_channel_msg = ExecuteMsg::ChannelCreate {
        salt: Binary::from("salt".as_bytes()),
        user_name: "Creator 2".to_string(),
        description: "Creator 2 description".to_string(),
        collabarators: None,
    };

    let res = app
        .execute_contract(
            creator2.clone(),
            channel_contract_addr.clone(),
            &create_channel_msg,
            &[],
        )
        .unwrap();

    // Get the channel_id from the event
    let creator2_channel_id = get_event_attribute(res.clone(), "wasm", "channel_id");

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

    // Publish the asset under creator 1's channel
    let publish_msg = ExecuteMsg::Publish {
        asset_onft_collection_id: asset_collection_id.clone(),
        asset_onft_id: asset_id.clone(),
        salt: Binary::from("salt".as_bytes()),
        channel_id: creator1_channel_id.clone(),
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

    // Create a playlist under creator 2's channel
    let create_playlist_msg = ExecuteMsg::PlaylistCreate {
        playlist_name: "Creator2 Playlist".to_string(),
        channel_id: creator2_channel_id.clone(),
    };

    let _res = app
        .execute_contract(
            creator2.clone(),
            channel_contract_addr.clone(),
            &create_playlist_msg,
            &[],
        )
        .unwrap();

    // Add an asset to the playlist
    let add_asset_msg = ExecuteMsg::PlaylistAddAsset {
        publish_id: publish_id.clone(),
        asset_channel_id: creator1_channel_id.clone(),
        channel_id: creator2_channel_id.clone(),
        playlist_name: "Creator2 Playlist".to_string(),
    };

    let _res = app
        .execute_contract(
            creator2.clone(),
            channel_contract_addr.clone(),
            &add_asset_msg,
            &[],
        )
        .unwrap();

    // Query the playlist
    let query_msg = QueryMsg::Playlist {
        channel_id: creator2_channel_id.clone(),
        playlist_name: "Creator2 Playlist".to_string(),
    };

    let playlist: Playlist = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();

    assert_eq!(playlist.assets.len(), 1);
    assert_eq!(playlist.assets[0].onft_id, asset_id.clone());
}
