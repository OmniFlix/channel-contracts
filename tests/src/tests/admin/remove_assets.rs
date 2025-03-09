use cosmwasm_std::{coin, Binary};
use cw_multi_test::Executor;
use omniflix_channel::ContractError;
use omniflix_channel_types::asset::{Asset, AssetSource};
use omniflix_channel_types::msg::{ExecuteMsg, QueryMsg};

use crate::helpers::msg_wrapper::CreateChannelMsgBuilder;
use crate::helpers::utils::get_event_attribute;
use crate::helpers::{msg_wrapper::get_channel_instantiate_msg, setup::setup};

#[test]
fn happy_path() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();
    let creator = setup_response.test_accounts.creator.clone();

    // Instantiate the contract
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

    let channel_id = get_event_attribute(res, "wasm", "channel_id");

    let publish_msg = ExecuteMsg::AssetPublish {
        asset_source: AssetSource::OffChain {
            media_uri: "https://example.com/media.png".to_string(),
            name: "Media Asset".to_string(),
            description: "This is a media asset".to_string(),
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

    // Admin removes the asset
    let remove_assets_msg = ExecuteMsg::AdminRemoveAssets {
        asset_keys: vec![(channel_id.clone(), publish_id.clone())],
        refresh_flags: None,
    };

    let _res = app
        .execute_contract(
            admin.clone(),
            channel_contract_addr.clone(),
            &remove_assets_msg,
            &[],
        )
        .unwrap();

    // Query the asset and check if it is removed
    let query_msg = QueryMsg::Assets {
        channel_id: channel_id.clone(),
        start_after: None,
        limit: None,
    };

    let assets: Vec<Asset> = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();

    assert!(assets.is_empty());
}

#[test]
fn unauthorized() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();
    let creator = setup_response.test_accounts.creator.clone();

    // Instantiate the contract
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

    let channel_id = get_event_attribute(res, "wasm", "channel_id");

    // Publish an asset
    let publish_msg = ExecuteMsg::AssetPublish {
        asset_source: AssetSource::OffChain {
            media_uri: "https://example.com/media.png".to_string(),
            name: "Media Asset".to_string(),
            description: "This is a media asset".to_string(),
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

    // Asset creator can not remove the asset with admin remove assets.
    let remove_assets_msg = ExecuteMsg::AdminRemoveAssets {
        asset_keys: vec![(channel_id.clone(), publish_id.clone())],
        refresh_flags: None,
    };

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &remove_assets_msg,
            &[],
        )
        .unwrap_err();
    let err = res.downcast_ref::<ContractError>().unwrap();

    assert_eq!(err, &ContractError::Unauthorized {});
}

#[test]
fn asset_does_not_exist() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();
    let creator = setup_response.test_accounts.creator.clone();
    // Instantiate the contract
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

    let channel_id = get_event_attribute(res, "wasm", "channel_id");

    // Admin removes the asset that does not exist
    let remove_assets_msg = ExecuteMsg::AdminRemoveAssets {
        asset_keys: vec![(channel_id.clone(), "publish_id".to_string())],
        refresh_flags: None,
    };

    let res = app
        .execute_contract(
            admin.clone(),
            channel_contract_addr.clone(),
            &remove_assets_msg,
            &[],
        )
        .unwrap_err();
    let err = res.downcast_ref::<ContractError>().unwrap();

    assert_eq!(
        err,
        &ContractError::Asset(asset_manager::error::AssetError::AssetNotFound {})
    );
}
