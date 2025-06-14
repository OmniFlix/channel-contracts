use asset_manager::error::PlaylistError;
use cosmwasm_std::{coin, Binary, CosmosMsg};
use cw_multi_test::Executor;
use omniflix_channel::ContractError;
use omniflix_channel_types::asset::{AssetSource, Playlist};
use omniflix_channel_types::msg::{AssetResponse, ExecuteMsg, QueryMsg};

use crate::helpers::msg_wrapper::AssetPublishMsgBuilder;
use crate::helpers::{
    msg_wrapper::{get_channel_instantiate_msg, CreateChannelMsgBuilder},
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
    let create_channel_msg = CreateChannelMsgBuilder::new("username", creator.clone()).build();

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
    let publish_msg = AssetPublishMsgBuilder::new(channel_id.clone())
        .asset_source(AssetSource::Nft {
            collection_id: "id".to_string(),
            onft_id: "asset_id".to_string(),
        })
        .build();

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
fn channel_not_owned() {
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
    let create_channel_msg = CreateChannelMsgBuilder::new("username", creator.clone()).build();

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
        asset_collection_id.clone(),
        asset_id.clone(),
        collector.clone().to_string(),
    );
    let cosmos_msg: CosmosMsg = mint_onft_msg;
    let _res = app.execute(collector.clone(), cosmos_msg);

    // Publish the asset
    let publish_msg = AssetPublishMsgBuilder::new(channel_id.clone())
        .asset_source(AssetSource::Nft {
            collection_id: asset_collection_id.clone(),
            onft_id: asset_id.clone(),
        })
        .build();

    let res = app
        .execute_contract(
            collector.clone(),
            channel_contract_addr.clone(),
            &publish_msg,
            &[],
        )
        .unwrap_err();

    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(typed_err, &ContractError::Unauthorized {});
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
    let create_channel_msg = CreateChannelMsgBuilder::new("username", creator.clone()).build();

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
        asset_collection_id.clone(),
        asset_id.clone(),
        creator.clone().to_string(),
    );
    let cosmos_msg: CosmosMsg = mint_onft_msg;
    let _res = app.execute(creator.clone(), cosmos_msg);

    // Publish the asset with wrong playlist name
    let publish_msg = AssetPublishMsgBuilder::new(channel_id.clone())
        .asset_source(AssetSource::Nft {
            collection_id: asset_collection_id.clone(),
            onft_id: asset_id.clone(),
        })
        .playlist_id("Wrong playlist".to_string())
        .build();

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
    let create_channel_msg = CreateChannelMsgBuilder::new("username", creator.clone()).build();

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
        asset_collection_id.clone(),
        asset_id.clone(),
        creator.clone().to_string(),
    );
    let cosmos_msg: CosmosMsg = mint_onft_msg;
    let _res = app.execute(creator.clone(), cosmos_msg);

    // Create a playlist
    let create_playlist_msg = ExecuteMsg::PlaylistCreate {
        playlist_name: "My Videos".to_string(),
        channel_id: channel_id.clone(),
        salt: Binary::from(b"salt1"),
    };

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &create_playlist_msg,
            &[],
        )
        .unwrap();

    let playlist_id = get_event_attribute(res.clone(), "wasm", "playlist_id");

    // Publish the asset under the new playlist
    let publish_msg = AssetPublishMsgBuilder::new(channel_id.clone())
        .asset_source(AssetSource::Nft {
            collection_id: asset_collection_id.clone(),
            onft_id: asset_id.clone(),
        })
        .playlist_id(playlist_id.clone())
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

    // Query the new playlist
    let query_msg = QueryMsg::Playlist {
        channel_id: channel_id.clone(),
        playlist_id: playlist_id.clone(),
    };

    let playlist: Playlist = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();

    assert_eq!(playlist.assets.len(), 1);
    assert_eq!(playlist.assets[0].1, publish_id);
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
    let create_channel_msg = CreateChannelMsgBuilder::new("username", creator.clone()).build();

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
        collector.clone().to_string(),
    );
    let _res = app.execute(creator.clone(), mint_onft_msg);

    // Asset is owned by collector
    // Creator tries to publish the asset
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

#[test]
fn publish_off_chain_asset() {
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
    let create_channel_msg = CreateChannelMsgBuilder::new("username", creator.clone()).build();

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

    let publish_msg = AssetPublishMsgBuilder::new(channel_id.clone())
        .asset_source(AssetSource::OffChain {})
        .name("name".to_string())
        .description("description".to_string())
        .media_uri("https://omniflix.network/".to_string())
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

    // Query the asset
    let query_msg = QueryMsg::Asset {
        channel_id: channel_id.clone(),
        publish_id: publish_id.clone(),
    };

    let asset: AssetResponse = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();

    assert_eq!(asset.asset.publish_id, publish_id);
    assert_eq!(asset.asset.asset_source, AssetSource::OffChain {});
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
    let create_channel_msg = CreateChannelMsgBuilder::new("username", creator.clone()).build();

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
    let cosmos_msg: CosmosMsg = mint_onft_msg;
    let _res = app.execute(creator.clone(), cosmos_msg);

    // Publish the asset
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

    // Query the asset
    let query_msg = QueryMsg::Asset {
        channel_id: channel_id.clone(),
        publish_id: publish_id.clone(),
    };

    let asset: AssetResponse = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();

    assert_eq!(asset.asset.publish_id, publish_id);
    assert_eq!(
        asset.asset.asset_source,
        AssetSource::Nft {
            collection_id: asset_collection_id.clone(),
            onft_id: asset_id.clone(),
        }
    );
}
