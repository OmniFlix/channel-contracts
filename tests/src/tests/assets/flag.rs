use cosmwasm_std::coin;
use cw_multi_test::Executor;
use omniflix_channel_types::{
    asset::{AssetSource, Flag},
    msg::ExecuteMsg,
};

use crate::helpers::msg_wrapper::AssetPublishMsgBuilder;
use crate::helpers::{
    msg_wrapper::{get_channel_instantiate_msg, CreateChannelMsgBuilder},
    setup::setup,
    utils::{create_denom_msg, get_event_attribute, mint_onft_msg},
};

#[test]
fn flag_asset_happy_path() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();
    let creator = setup_response.test_accounts.creator.clone();

    // Instantiate Channel Contract
    let channel_contract_addr = app
        .instantiate_contract(
            setup_response.channel_contract_code_id,
            admin.clone(),
            &get_channel_instantiate_msg(admin.clone()),
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
    let channel_id = get_event_attribute(res.clone(), "wasm", "channel_id");

    // Create an asset for the channel
    let collection_id = "collection1";
    let asset_id = "asset1";

    // Create collection and mint asset
    let create_denom_msg = create_denom_msg(
        creator.clone().to_string(),
        collection_id.to_string(),
        Some("Test Collection".to_string()),
    );
    let _res = app.execute(creator.clone(), create_denom_msg);

    let mint_asset_msg = mint_onft_msg(
        collection_id.to_string(),
        asset_id.to_string(),
        creator.clone().to_string(),
    );
    let _res = app.execute(creator.clone(), mint_asset_msg);

    // Publish the asset
    let publish_msg = AssetPublishMsgBuilder::new(channel_id.clone())
        .asset_source(AssetSource::Nft {
            collection_id: collection_id.to_string(),
            onft_id: asset_id.to_string(),
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

    // User flags the asset
    let flag_msg = ExecuteMsg::AssetFlag {
        channel_id: channel_id.clone(),
        publish_id: publish_id.clone(),
        flag: Flag::NSFW,
        interactive_video_id: None,
    };

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &flag_msg,
            &[],
        )
        .unwrap();

    // Verify correct event attributes
    assert_eq!(
        get_event_attribute(res.clone(), "wasm", "action"),
        "asset_flag"
    );
    assert_eq!(
        get_event_attribute(res.clone(), "wasm", "publish_id"),
        publish_id
    );
    assert_eq!(
        get_event_attribute(res.clone(), "wasm", "channel_id"),
        channel_id
    );
    assert_eq!(get_event_attribute(res.clone(), "wasm", "flag"), "NSFW");
}
