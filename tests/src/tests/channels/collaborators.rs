use channel_manager::error::ChannelError;
use cosmwasm_std::{coin, Decimal, Uint128};
use cw_multi_test::Executor;
use omniflix_channel::ContractError;
use omniflix_channel_types::channel::{ChannelCollaborator, Role};
use omniflix_channel_types::msg::{CollaboratorInfo, ExecuteMsg, QueryMsg};

use crate::helpers::{
    msg_wrapper::{get_channel_instantiate_msg, CreateChannelMsgBuilder},
    setup::setup,
    utils::get_event_attribute,
};

#[test]
fn add_collaborator_unauthorized() {
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

    // Try to add collaborator as unauthorized user
    let msg = ExecuteMsg::ChannelAddCollaborator {
        channel_id: channel_id.clone(),
        collaborator_address: creator.clone().into_string(),
        collaborator_details: ChannelCollaborator {
            role: Role::Moderator,
            share: Decimal::from_ratio(Uint128::one(), Uint128::from(3u128)),
        },
    };

    let res = app
        .execute_contract(collector.clone(), channel_contract_addr.clone(), &msg, &[])
        .unwrap_err();

    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(typed_err, &ContractError::Unauthorized {});
}

#[test]
fn add_collaborator_invalid_share() {
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

    // Try to add collaborator with share > 100%
    let msg = ExecuteMsg::ChannelAddCollaborator {
        channel_id: channel_id.clone(),
        collaborator_address: collector.clone().into_string(),
        collaborator_details: ChannelCollaborator {
            role: Role::Moderator,
            share: Decimal::percent(101),
        },
    };

    let res = app
        .execute_contract(creator.clone(), channel_contract_addr.clone(), &msg, &[])
        .unwrap_err();

    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::Channel(ChannelError::InvalidSharePercentage {})
    );
}

#[test]
fn add_collaborator_total_unique_limit_exceeded() {
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

    let channel_id = get_event_attribute(res, "wasm", "channel_id");

    // Need to add 10 collaborators
    for i in 0..10 {
        // Generate a valid cosmos collaborator address
        let collaborator_address = app.api().addr_make(format!("collaborator_{}", i).as_str());

        let msg = ExecuteMsg::ChannelAddCollaborator {
            channel_id: channel_id.clone(),
            collaborator_address: collaborator_address.clone().into_string(),
            collaborator_details: ChannelCollaborator {
                role: Role::Moderator,
                share: Decimal::percent(9),
            },
        };

        let _res = app
            .execute_contract(creator.clone(), channel_contract_addr.clone(), &msg, &[])
            .unwrap();
    }

    // Try to add another collaborator should fail
    let msg = ExecuteMsg::ChannelAddCollaborator {
        channel_id: channel_id.clone(),
        collaborator_address: app.api().addr_make("new_collaborator").into_string(),
        collaborator_details: ChannelCollaborator {
            role: Role::Moderator,
            share: Decimal::percent(9),
        },
    };

    let res = app
        .execute_contract(creator.clone(), channel_contract_addr.clone(), &msg, &[])
        .unwrap_err();

    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::Channel(ChannelError::TotalUniqueCollaboratorsLimitExceeded {})
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

    // Add collaborator successfully
    let msg = ExecuteMsg::ChannelAddCollaborator {
        channel_id: channel_id.clone(),
        collaborator_address: collector.clone().into_string(),
        collaborator_details: ChannelCollaborator {
            role: Role::Moderator,
            share: Decimal::percent(30),
        },
    };

    let _res = app
        .execute_contract(creator.clone(), channel_contract_addr.clone(), &msg, &[])
        .unwrap();

    // Verify collaborator was added
    let query_msg = QueryMsg::GetChannelCollaborator {
        channel_id: channel_id.clone(),
        collaborator_address: collector.clone(),
    };

    let collaborator: CollaboratorInfo = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();

    assert_eq!(collaborator.role, Role::Moderator.to_string());
    assert_eq!(collaborator.share, Decimal::percent(30));
}
