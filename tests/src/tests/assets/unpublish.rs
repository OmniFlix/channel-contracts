use cosmwasm_std::{coin, Binary};
use cw_multi_test::Executor;
use omniflix_channel::ContractError;
use omniflix_channel_types::asset::{Asset, AssetSource};
use omniflix_channel_types::msg::{ExecuteMsg, QueryMsg};

use crate::helpers::{
    msg_wrapper::{get_channel_instantiate_msg, CreateChannelMsgBuilder},
    setup::setup,
    utils::{create_denom_msg, get_event_attribute, mint_onft_msg},
};

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

    // Create an asset
    let asset_collection_id = "id".to_string();
    let asset_id = "asset_id".to_string();

    // Create a collection
    let create_denom_msg = create_denom_msg(
        creator.clone().to_string(),
        asset_collection_id.clone(),
        Some("Asset Collection".to_string()),
    );
    let _res = app.execute(creator.clone(), create_denom_msg);

    let mint_onft_msg = mint_onft_msg(
        asset_collection_id.clone(),
        asset_id.clone(),
        creator.clone().to_string(),
    );

    let _res = app.execute(creator.clone(), mint_onft_msg);

    // Publish an asset
    let publish_msg = ExecuteMsg::Publish {
        asset_source: AssetSource::Nft {
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

    // Get the publish_id from the event
    let publish_id = get_event_attribute(res.clone(), "wasm", "publish_id");

    // Unpublish the asset with a different user
    let unpublish_msg = ExecuteMsg::Unpublish {
        publish_id: publish_id.clone(),
        channel_id: channel_id.clone(),
    };

    let res = app
        .execute_contract(
            collector.clone(),
            channel_contract_addr.clone(),
            &unpublish_msg,
            &[],
        )
        .unwrap_err();

    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(typed_err, &ContractError::Unauthorized {});
}

#[test]
fn asset_not_pubished() {
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

    // Unpublish the asset
    // Should return an error
    let unpublish_msg = ExecuteMsg::Unpublish {
        publish_id: "publish_id".to_string(),
        channel_id: channel_id.clone(),
    };

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &unpublish_msg,
            &[],
        )
        .unwrap_err();

    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::Asset(asset_manager::error::AssetError::AssetNotFound {})
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

    // Create an asset
    let asset_collection_id = "id".to_string();
    let asset_id = "asset_id".to_string();

    // Create a collection
    let create_denom_msg = create_denom_msg(
        creator.clone().to_string(),
        asset_collection_id.clone(),
        Some("Asset Collection".to_string()),
    );
    let _res = app.execute(creator.clone(), create_denom_msg);

    let mint_onft_msg = mint_onft_msg(
        asset_collection_id.clone(),
        asset_id.clone(),
        creator.clone().to_string(),
    );

    let _res = app.execute(creator.clone(), mint_onft_msg);

    // Publish an asset
    let publish_msg = ExecuteMsg::Publish {
        asset_source: AssetSource::Nft {
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

    // Get the publish_id from the event
    let publish_id = get_event_attribute(res.clone(), "wasm", "publish_id");

    // Query the asset and check if it is published
    let query_msg = QueryMsg::Assets {
        channel_id: channel_id.clone(),
        start_after: None,
        limit: None,
    };

    let assets: Vec<Asset> = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();

    let asset = assets.first().unwrap();
    assert_eq!(asset.publish_id, publish_id.clone());

    // Unpublish the asset
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

    // Query the asset and check if it is unpublished
    let query_msg = QueryMsg::Assets {
        channel_id: channel_id.clone(),
        start_after: None,
        limit: None,
    };

    let assets: Vec<Asset> = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();

    assert_eq!(assets.len(), 0);
}
