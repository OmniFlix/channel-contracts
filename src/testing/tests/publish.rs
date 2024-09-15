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

    // We need to create a denom for creator
    // Then we will mint a onft representing the asset
    // Then we will publish the asset

    // let cosmos_msg: CosmosMsg = create_denom_msg.into();
    let create_denom_msg = create_denom_msg(
        creator.clone().to_string(),
        "id".to_string(),
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
}
