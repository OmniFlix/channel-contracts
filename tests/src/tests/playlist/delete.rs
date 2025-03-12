use crate::helpers::msg_wrapper::{get_channel_instantiate_msg, CreateChannelMsgBuilder};
use crate::helpers::setup::setup;
use crate::helpers::utils::{create_denom_msg, get_event_attribute, mint_onft_msg};
use asset_manager::error::PlaylistError;
use cosmwasm_std::{coin, Binary, CosmosMsg};
use cw_multi_test::Executor;
use omniflix_channel::ContractError;
use omniflix_channel_types::asset::{AssetSource, Playlist};
use omniflix_channel_types::msg::{ExecuteMsg, QueryMsg};

#[test]
fn does_not_exist() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();
    let creator = setup_response.test_accounts.creator.clone();

    // Instantiate Channel Contract
    let instantiate_msg = get_channel_instantiate_msg(admin.clone());

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
    let create_channel_msg = CreateChannelMsgBuilder::new("creator", creator.clone()).build();

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

    // Remove a playlist that does not exist
    let delete_playlist_msg = ExecuteMsg::PlaylistDelete {
        playlist_name: "My Playlist".to_string(),
        channel_id: channel_id.clone(),
    };

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &delete_playlist_msg,
            &[],
        )
        .unwrap_err();

    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::Playlist(PlaylistError::PlaylistNotFound {})
    );
}

// Delete a playlist without owning the channel
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
    let instantiate_msg = get_channel_instantiate_msg(admin.clone());

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
    let create_channel_msg = CreateChannelMsgBuilder::new("creator", creator.clone()).build();

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

    // Remove a playlist without owning the channel
    let delete_playlist_msg = ExecuteMsg::PlaylistDelete {
        playlist_name: "My Playlist".to_string(),
        channel_id: channel_id.clone(),
    };

    let res = app
        .execute_contract(
            collector.clone(),
            channel_contract_addr.clone(),
            &delete_playlist_msg,
            &[],
        )
        .unwrap_err();

    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(typed_err, &ContractError::Unauthorized {});
}

// Happy path
#[test]
fn happy_path() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();
    let creator = setup_response.test_accounts.creator.clone();

    // Instantiate Channel Contract
    let instantiate_msg = get_channel_instantiate_msg(admin.clone());

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
    let create_channel_msg = CreateChannelMsgBuilder::new("creator", creator.clone()).build();

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

    let publish_msg = ExecuteMsg::AssetPublish {
        asset_source: AssetSource::Nft {
            collection_id: asset_collection_id.clone(),
            onft_id: asset_id.clone(),
            name: "name".to_string(),
            description: "description".to_string(),
            media_uri: "http://www.media.com".to_string(),
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
    let publish_id = get_event_attribute(res, "wasm", "publish_id");

    // Playlist Add Asset
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
    assert_eq!(playlist.assets[0].1, publish_id.clone());

    // Delete the playlist
    let delete_playlist_msg = ExecuteMsg::PlaylistDelete {
        playlist_name: "My Playlist".to_string(),
        channel_id: channel_id.clone(),
    };

    let _res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &delete_playlist_msg,
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

    assert_eq!(playlists.len(), 0);
}
