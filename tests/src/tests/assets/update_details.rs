use asset_manager::error::AssetError;
use cosmwasm_std::coin;
use cw_multi_test::Executor;
use omniflix_channel::ContractError;
use omniflix_channel_types::channel::Role;
use omniflix_channel_types::msg::{AssetResponse, ExecuteMsg, QueryMsg};

use crate::helpers::msg_wrapper::AssetPublishMsgBuilder;
use crate::helpers::{
    msg_wrapper::{get_channel_instantiate_msg, CreateChannelMsgBuilder},
    setup::setup,
    utils::get_event_attribute,
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
    let channel_id = get_event_attribute(res, "wasm", "channel_id");

    // Try to update details of a non-existent asset
    let update_details_msg = ExecuteMsg::AssetUpdateDetails {
        publish_id: "non_existent_publish_id".to_string(),
        channel_id: channel_id.clone(),
        is_visible: Some(false),
        name: Some("Updated Name".to_string()),
        description: Some("Updated Description".to_string()),
        media_uri: Some("https://updated-media-uri.com".to_string()),
        thumbnail_uri: None,
    };

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &update_details_msg,
            &[],
        )
        .unwrap_err();
    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::Asset(AssetError::AssetNotFound {})
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
    let other_user = setup_response.test_accounts.creator2.clone();

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
    let channel_id = get_event_attribute(res, "wasm", "channel_id");

    // Publish an asset
    let publish_msg = AssetPublishMsgBuilder::new(channel_id.clone()).build();

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &publish_msg,
            &[],
        )
        .unwrap();

    // Get the publish_id from the event
    let publish_id = get_event_attribute(res, "wasm", "publish_id");

    // Try to update asset details from a non-owner account
    let update_details_msg = ExecuteMsg::AssetUpdateDetails {
        publish_id: publish_id.clone(),
        channel_id: channel_id.clone(),
        is_visible: Some(false),
        name: Some("Updated Name".to_string()),
        description: Some("Updated Description".to_string()),
        media_uri: Some("https://updated-media-uri.com".to_string()),
        thumbnail_uri: None,
    };

    let err = app
        .execute_contract(
            other_user.clone(),
            channel_contract_addr.clone(),
            &update_details_msg,
            &[],
        )
        .unwrap_err();

    let typed_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(typed_err, &ContractError::Unauthorized {});
}

#[test]
fn partial_update() {
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
    let channel_id = get_event_attribute(res, "wasm", "channel_id");

    // Publish an asset
    let original_name = "Original Asset Name";
    let original_description = "Original Asset Description";
    let original_media_uri = "https://original-media-uri.com";

    let publish_msg = AssetPublishMsgBuilder::new(channel_id.clone())
        .name(original_name.to_string())
        .description(original_description.to_string())
        .media_uri(original_media_uri.to_string())
        .build();

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &publish_msg,
            &[],
        )
        .unwrap();

    // Get the publish_id from the event
    let publish_id = get_event_attribute(res, "wasm", "publish_id");

    // Update only the name of the asset
    let new_name = "Updated Asset Name";
    let update_details_msg = ExecuteMsg::AssetUpdateDetails {
        publish_id: publish_id.clone(),
        channel_id: channel_id.clone(),
        is_visible: None,
        name: Some(new_name.to_string()),
        description: None,
        media_uri: None,
        thumbnail_uri: None,
    };

    app.execute_contract(
        creator.clone(),
        channel_contract_addr.clone(),
        &update_details_msg,
        &[],
    )
    .unwrap();

    // Query the asset to verify the update
    let query_msg = QueryMsg::Asset {
        channel_id: channel_id.clone(),
        publish_id: publish_id.clone(),
    };

    let asset_response: AssetResponse = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();

    // Verify that only the name was updated
    assert_eq!(asset_response.metadata.name, new_name);
    assert_eq!(asset_response.metadata.description, original_description);
    assert_eq!(asset_response.metadata.media_uri, original_media_uri);
    assert!(asset_response.asset.is_visible);
}

#[test]
fn full_update() {
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
    let channel_id = get_event_attribute(res, "wasm", "channel_id");

    // Publish an asset
    let original_name = "Original Asset Name";
    let original_description = "Original Asset Description";
    let original_media_uri = "https://original-media-uri.com";
    let original_thumbnail_uri = "https://original-thumbnail-uri.com";

    let publish_msg = AssetPublishMsgBuilder::new(channel_id.clone())
        .name(original_name.to_string())
        .description(original_description.to_string())
        .media_uri(original_media_uri.to_string())
        .thumbnail_uri(original_thumbnail_uri.to_string())
        .build();

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &publish_msg,
            &[],
        )
        .unwrap();

    // Get the publish_id from the event
    let publish_id = get_event_attribute(res, "wasm", "publish_id");

    // Update all fields of the asset
    let new_name = "Completely Updated Name";
    let new_description = "Completely Updated Description";
    let new_media_uri = "https://completely-updated-media-uri.com";
    let new_thumbnail_uri = "https://completely-updated-thumbnail-uri.com";
    let update_details_msg = ExecuteMsg::AssetUpdateDetails {
        publish_id: publish_id.clone(),
        channel_id: channel_id.clone(),
        is_visible: Some(false),
        name: Some(new_name.to_string()),
        description: Some(new_description.to_string()),
        media_uri: Some(new_media_uri.to_string()),
        thumbnail_uri: Some(new_thumbnail_uri.to_string()),
    };

    app.execute_contract(
        creator.clone(),
        channel_contract_addr.clone(),
        &update_details_msg,
        &[],
    )
    .unwrap();

    // Query the asset to verify the update
    let query_msg = QueryMsg::Asset {
        channel_id: channel_id.clone(),
        publish_id: publish_id.clone(),
    };

    let asset_response: AssetResponse = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();

    // Verify that all fields were updated
    assert_eq!(asset_response.metadata.name, new_name);
    assert_eq!(asset_response.metadata.description, new_description);
    assert_eq!(asset_response.metadata.media_uri, new_media_uri);
    assert_eq!(
        asset_response.metadata.thumbnail_uri,
        Some(new_thumbnail_uri.to_string())
    );
    assert!(!asset_response.asset.is_visible);
}

#[test]
fn collaborator_update() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();
    let creator = setup_response.test_accounts.creator.clone();
    let collaborator = setup_response.test_accounts.collaborator.clone();

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
    let channel_id = get_event_attribute(res, "wasm", "channel_id");

    // Add collaborator to the channel
    let add_collaborator_msg = ExecuteMsg::ChannelAddCollaborator {
        channel_id: channel_id.clone(),
        collaborator_address: collaborator.to_string(),
        collaborator_details: omniflix_channel_types::channel::ChannelCollaborator {
            role: Role::Publisher,
            share: cosmwasm_std::Decimal::percent(10),
        },
    };

    app.execute_contract(
        creator.clone(),
        channel_contract_addr.clone(),
        &add_collaborator_msg,
        &[],
    )
    .unwrap();

    // Publish an asset
    let original_name = "Original Asset Name";
    let original_description = "Original Asset Description";
    let original_media_uri = "https://original-media-uri.com";

    let publish_msg = AssetPublishMsgBuilder::new(channel_id.clone())
        .name(original_name.to_string())
        .description(original_description.to_string())
        .media_uri(original_media_uri.to_string())
        .build();

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &publish_msg,
            &[],
        )
        .unwrap();

    // Get the publish_id from the event
    let publish_id = get_event_attribute(res, "wasm", "publish_id");

    // Collaborator updates the asset details
    let new_name = "Collaborator Updated Name";
    let update_details_msg = ExecuteMsg::AssetUpdateDetails {
        publish_id: publish_id.clone(),
        channel_id: channel_id.clone(),
        is_visible: Some(false),
        name: Some(new_name.to_string()),
        description: None,
        media_uri: None,
        thumbnail_uri: None,
    };

    // This should succeed since collaborators should be able to update assets
    app.execute_contract(
        collaborator.clone(),
        channel_contract_addr.clone(),
        &update_details_msg,
        &[],
    )
    .unwrap();

    // Query the asset to verify the update
    let query_msg = QueryMsg::Asset {
        channel_id: channel_id.clone(),
        publish_id: publish_id.clone(),
    };

    let asset_response: AssetResponse = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();

    // Verify that the name was updated and visibility changed
    assert_eq!(asset_response.metadata.name, new_name);
    assert_eq!(asset_response.metadata.description, original_description);
    assert_eq!(asset_response.metadata.media_uri, original_media_uri);
    assert!(!asset_response.asset.is_visible);
}
