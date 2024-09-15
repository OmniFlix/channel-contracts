use crate::msg::{ExecuteMsg, QueryMsg};
use crate::playlist::Playlist;
use crate::testing::utils::{create_denom_msg, get_event_attribute, mint_onft_msg};
use crate::ContractError;
use crate::{msg::InstantiateMsg, testing::setup::setup};
use cosmwasm_std::{coin, Binary, CosmosMsg};
use cw_multi_test::Executor;

#[test]
fn create_playlist() {
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
    let channel_onft_id = get_event_attribute(res.clone(), "wasm", "onft_id");

    // Create a playlist that already exists
    let create_playlist_msg = ExecuteMsg::CreatePlaylist {
        playlist_name: "My Videos".to_string(),
        channel_id: channel_id.clone(),
    };

    let res = app
        .execute_contract(
            creator.clone(),
            channel_contract_addr.clone(),
            &create_playlist_msg,
            &[],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let typed_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(typed_err, &ContractError::PlaylistAlreadyExists {});

    // Create a playlist without owning the channel
    let create_playlist_msg = ExecuteMsg::CreatePlaylist {
        playlist_name: "My Playlist".to_string(),
        channel_id: channel_id.clone(),
    };

    let res = app
        .execute_contract(
            collector.clone(),
            channel_contract_addr.clone(),
            &create_playlist_msg,
            &[],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let typed_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::OnftNotOwned {
            onft_id: channel_onft_id.clone(),
            collection_id: "Channels".to_string(),
        }
    );
}
