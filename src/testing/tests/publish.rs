use crate::channels::ChannelDetails;
use crate::helpers::generate_random_id_with_prefix;
use crate::msg::{ExecuteMsg, QueryMsg};
use crate::playlist::Playlist;
use crate::testing::utils::{create_denom_msg, get_event_attribute, mint_onft_msg};
use crate::ContractError;
use crate::{msg::InstantiateMsg, testing::setup::setup};
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{coin, Binary, BlockInfo, CosmosMsg, Timestamp};
use cw_multi_test::Executor;

#[test]
fn publish_asset() {
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
    };

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &publish_msg,
            &[],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let typed_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(typed_err, &ContractError::OnftNotFound {});

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
        "id".to_string(),
        "asset_id".to_string(),
        creator.clone().to_string(),
    );
    let cosmos_msg: CosmosMsg = mint_onft_msg.into();
    let _res = app.execute(creator.clone(), cosmos_msg);

    // Publish the asset without owning it
    // This should fail because actor "collector" does not own the asset
    let publish_msg = ExecuteMsg::Publish {
        asset_onft_collection_id: asset_collection_id.clone(),
        asset_onft_id: asset_id.clone(),
        salt: Binary::from("salt".as_bytes()),
        channel_id: channel_id.clone(),
        playlist_name: None,
    };

    let res = app
        .execute_contract(
            collector.clone(),
            channel_contract_addr.clone(),
            &publish_msg,
            &[],
        )
        .unwrap_err();

    let err = res.source().unwrap();
    let typed_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(typed_err, &ContractError::ChannelNotOwned {});

    // Publish the asset
    let publish_msg = ExecuteMsg::Publish {
        asset_onft_collection_id: asset_collection_id.clone(),
        asset_onft_id: asset_id.clone(),
        salt: Binary::from("salt".as_bytes()),
        channel_id: channel_id.clone(),
        playlist_name: None,
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
    let query_msg = QueryMsg::Playlist {
        channel_id: channel_id.clone(),
        playlist_name: "My Videos".to_string(), // Default playlist name
    };

    let playlist: Playlist = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();

    assert_eq!(playlist.assets.len(), 1);
    assert_eq!(playlist.assets[0].publish_id, publish_id);
}

#[test]
fn publish_asset_without_owning_the_channel() {
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
    };

    let res = app
        .execute_contract(
            collector.clone(),
            channel_contract_addr.clone(),
            &publish_msg,
            &[],
        )
        .unwrap_err();

    let err = res.source().unwrap();
    let typed_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(typed_err, &ContractError::ChannelNotOwned {});
}
