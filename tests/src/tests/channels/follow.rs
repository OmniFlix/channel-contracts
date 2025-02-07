use crate::helpers::msg_wrapper::{get_channel_instantiate_msg, CreateChannelMsgBuilder};
use crate::helpers::setup::setup;
use crate::helpers::utils::get_event_attribute;
use channel_manager::error::ChannelError;
use cosmwasm_std::{coin, Addr};
use cw_multi_test::Executor;
use omniflix_channel::ContractError;
use omniflix_channel_types::msg::{ExecuteMsg, QueryMsg};

#[test]
fn follow_non_existent_channel() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();
    let follower = setup_response.test_accounts.collector.clone();

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

    let res = app.execute_contract(
        follower.clone(),
        channel_contract_addr.clone(),
        &ExecuteMsg::ChannelFollow {
            channel_id: "non_existent_channel".to_string(),
        },
        &[],
    );

    assert!(res.is_err());
}
#[test]
fn already_following() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();
    let creator = setup_response.test_accounts.creator.clone();
    let follower = setup_response.test_accounts.collector.clone();

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

    let channel_id = get_event_attribute(res, "wasm", "channel_id");

    // Follow the channel
    let _res = app
        .execute_contract(
            follower.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::ChannelFollow {
                channel_id: channel_id.clone(),
            },
            &[],
        )
        .unwrap();

    // Try to follow again
    let res = app
        .execute_contract(
            follower.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::ChannelFollow {
                channel_id: channel_id.clone(),
            },
            &[],
        )
        .unwrap_err();

    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::Channel(ChannelError::AlreadyFollowing {})
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
    let follower = setup_response.test_accounts.collector.clone();

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

    let channel_id = get_event_attribute(res, "wasm", "channel_id");

    // Follow the channel
    let _res = app
        .execute_contract(
            follower.clone(),
            channel_contract_addr.clone(),
            &ExecuteMsg::ChannelFollow {
                channel_id: channel_id.clone(),
            },
            &[],
        )
        .unwrap();

    // Query followers count
    let followers_count: u64 = app
        .wrap()
        .query_wasm_smart(
            channel_contract_addr.clone(),
            &QueryMsg::FollowersCount {
                channel_id: channel_id.clone(),
            },
        )
        .unwrap();

    assert_eq!(followers_count, 1);

    // Query followers
    let followers: Vec<Addr> = app
        .wrap()
        .query_wasm_smart(
            channel_contract_addr.clone(),
            &QueryMsg::Followers {
                channel_id: channel_id.clone(),
                start_after: None,
                limit: None,
            },
        )
        .unwrap();

    assert_eq!(followers.len(), 1);
    assert_eq!(followers[0], follower);
}
