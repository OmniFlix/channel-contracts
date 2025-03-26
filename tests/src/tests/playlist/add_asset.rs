use cosmwasm_std::{coin, BlockInfo, Timestamp};
use cw_multi_test::Executor;
use omniflix_channel::ContractError;
use omniflix_channel_types::asset::{AssetSource, Playlist};
use omniflix_channel_types::msg::{ExecuteMsg, QueryMsg};

use crate::helpers::msg_wrapper::{
    get_channel_instantiate_msg, AssetPublishMsgBuilder, CreateChannelMsgBuilder,
};
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
        asset_collection_id.clone(),
        asset_id.clone(),
        creator.clone().to_string(),
    );
    let _res = app.execute(creator.clone(), mint_onft_msg);

    let publish_msg = AssetPublishMsgBuilder::new(channel_id.clone())
        .asset_source(AssetSource::Nft {
            collection_id: asset_collection_id.clone(),
            onft_id: asset_id.clone(),
        })
        .build();

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

    // Creator 1 creates a channel
    let create_channel_msg = CreateChannelMsgBuilder::new("creatorone", creator.clone())
        .description("Creator 1 Description".to_string())
        .channel_name("Creator1".to_string())
        .build();

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

    let create_channel_msg = CreateChannelMsgBuilder::new("creatortwo", creator2.clone())
        .description("Creator 2 Description".to_string())
        .channel_name("Creator2".to_string())
        .build();

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

    let publish_msg = AssetPublishMsgBuilder::new(creator1_channel_id.clone())
        .asset_source(AssetSource::Nft {
            collection_id: asset_collection_id.clone(),
            onft_id: asset_id.clone(),
        })
        .build();

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
    assert_eq!(playlist.assets[0].1, publish_id);
}

#[test]
fn asset_already_exists_in_playlist() {
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
        asset_collection_id.clone(),
        asset_id.clone(),
        creator.clone().to_string(),
    );
    let _res = app.execute(creator.clone(), mint_onft_msg);

    let publish_msg = AssetPublishMsgBuilder::new(channel_id.clone())
        .asset_source(AssetSource::Nft {
            collection_id: asset_collection_id.clone(),
            onft_id: asset_id.clone(),
        })
        .build();

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &publish_msg,
            &[],
        )
        .unwrap();

    let publish_id = get_event_attribute(res.clone(), "wasm", "publish_id");

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

    // Try to add the same asset again
    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &add_asset_msg,
            &[],
        )
        .unwrap_err();

    // Verify error is about asset already in playlist
    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::Playlist(
            asset_manager::error::PlaylistError::AssetAlreadyExistsInPlaylist {}
        )
    );
}

#[test]
fn playlist_asset_limit_reached() {
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

    // Create and publish 101 assets (1 more than the limit)
    // The constant PLAYLISTS_ASSET_LIMIT is typically set to 100
    for i in 0..101 {
        let publish_msg = AssetPublishMsgBuilder::new(channel_id.clone()).build();
        // Increase block time by 1 ns every iteration
        app.update_block(|block| {
            block.time = block.time.plus_nanos(1);
        });

        let res = app
            .execute_contract(
                creator.clone(),
                channel_contract_addr.clone(),
                &publish_msg,
                &[],
            )
            .unwrap();

        let publish_id = get_event_attribute(res.clone(), "wasm", "publish_id");

        // Add assets to the playlist (first 100 should succeed)
        let add_asset_msg = ExecuteMsg::PlaylistAddAsset {
            publish_id: publish_id.clone(),
            asset_channel_id: channel_id.clone(),
            channel_id: channel_id.clone(),
            playlist_name: "My Playlist".to_string(),
        };

        // The first 100 adds should succeed, the 101st should fail
        if i < 100 {
            let _res = app
                .execute_contract(
                    creator.clone(),
                    channel_contract_addr.clone(),
                    &add_asset_msg,
                    &[],
                )
                .unwrap();
        } else {
            // The 101st add should fail with PlaylistAssetLimitReached
            let res = app
                .execute_contract(
                    creator.clone(),
                    channel_contract_addr.clone(),
                    &add_asset_msg,
                    &[],
                )
                .unwrap_err();

            // Verify error is about playlist asset limit reached
            let typed_err = res.downcast_ref::<ContractError>().unwrap();
            assert_eq!(
                typed_err,
                &ContractError::Playlist(
                    asset_manager::error::PlaylistError::PlaylistAssetLimitReached {}
                )
            );
        }
    }
}
