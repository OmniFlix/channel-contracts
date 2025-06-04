use cosmwasm_std::{coin, Addr};
use cw_multi_test::Executor;
use omniflix_channel::string_validation::StringValidationError;
use omniflix_channel::ContractError;
use omniflix_channel_types::msg::{ExecuteMsg, QueryMsg, ReservedUsername};

use crate::helpers::msg_wrapper::get_channel_instantiate_msg;
use crate::helpers::setup::setup;

#[test]
fn add_reserved_usernames() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();
    let creator = setup_response.test_accounts.creator.clone();

    let mut instantiate_msg = get_channel_instantiate_msg(admin.clone());
    instantiate_msg.channel_creation_fee = vec![coin(1000000, "uflix")];

    // Instantiate the contract
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
    // Query reserved_usernames
    // Default reserved usernames has 1 entry
    let query_msg = QueryMsg::ReservedUsernames {
        limit: None,
        start_after: None,
    };
    let res: Vec<ReservedUsername> = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();
    assert_eq!(res.len(), 1);

    // Add a reserved username but dont set an address
    let msg = ExecuteMsg::AdminManageReservedUsernames {
        add_usernames: Some(vec![ReservedUsername {
            username: "admin".to_string(),
            address: None,
        }]),
        remove_usernames: None,
    };

    let _res = app
        .execute_contract(admin.clone(), channel_contract_addr.clone(), &msg, &[])
        .unwrap();

    // Query reserved_usernames
    let query_msg = QueryMsg::ReservedUsernames {
        limit: None,
        start_after: None,
    };
    let res: Vec<ReservedUsername> = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();
    assert_eq!(res.len(), 2);

    // Add invalid reserved username
    let msg = ExecuteMsg::AdminManageReservedUsernames {
        add_usernames: Some(vec![ReservedUsername {
            username: "Ad".to_string(),
            address: None,
        }]),
        remove_usernames: None,
    };

    let res = app
        .execute_contract(admin.clone(), channel_contract_addr.clone(), &msg, &[])
        .unwrap_err();

    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::StringValidationError(StringValidationError::InvalidLength {
            sent: "Ad".to_string(),
            min_length: 3,
            max_length: 32,
        })
    );
    // Add valid reserved username with invalid address
    let msg = ExecuteMsg::AdminManageReservedUsernames {
        add_usernames: Some(vec![ReservedUsername {
            username: "Admin".to_string(),
            address: Some(Addr::unchecked("")),
        }]),
        remove_usernames: None,
    };

    let _res = app
        .execute_contract(admin.clone(), channel_contract_addr.clone(), &msg, &[])
        .unwrap_err();
    // Username admin is already added but we want to sent an valid address
    let msg = ExecuteMsg::AdminManageReservedUsernames {
        add_usernames: Some(vec![ReservedUsername {
            username: "admin".to_string(),
            address: Some(creator.clone()),
        }]),
        remove_usernames: None,
    };

    let _res = app
        .execute_contract(admin.clone(), channel_contract_addr.clone(), &msg, &[])
        .unwrap();
    // Query reserved_usernames
    let query_msg = QueryMsg::ReservedUsernames {
        limit: None,
        start_after: None,
    };

    let res: Vec<ReservedUsername> = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();

    assert_eq!(res.len(), 2);
    assert_eq!(res[0].address, Some(creator.clone()));

    // Remove designated address from reserved username
    let msg = ExecuteMsg::AdminManageReservedUsernames {
        add_usernames: Some(vec![ReservedUsername {
            username: "admin".to_string(),
            address: None,
        }]),
        remove_usernames: None,
    };

    let _res = app
        .execute_contract(admin.clone(), channel_contract_addr.clone(), &msg, &[])
        .unwrap();

    // Query reserved_usernames
    let query_msg = QueryMsg::ReservedUsernames {
        limit: None,
        start_after: None,
    };
    let res: Vec<ReservedUsername> = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();
    assert_eq!(res.len(), 2);
    assert_eq!(res[0].address, None);
}

#[test]
fn remove_reserved_usernames() {
    // Setup testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();
    let _creator = setup_response.test_accounts.creator.clone();

    let mut instantiate_msg = get_channel_instantiate_msg(admin.clone());
    instantiate_msg.channel_creation_fee = vec![coin(1000000, "uflix")];

    // Instantiate the contract
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
    // Query reserved_usernames
    // Default reserved usernames has 1 entry
    let query_msg = QueryMsg::ReservedUsernames {
        limit: None,
        start_after: None,
    };
    let res: Vec<ReservedUsername> = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();
    assert_eq!(res.len(), 1);

    // Remove a reserved username that does not exist
    let msg = ExecuteMsg::AdminManageReservedUsernames {
        add_usernames: None,
        remove_usernames: vec!["admin".to_string()].into(),
    };

    let res = app
        .execute_contract(admin.clone(), channel_contract_addr.clone(), &msg, &[])
        .unwrap_err();

    let typed_err = res.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::Channel(channel_manager::error::ChannelError::UsernameNotReserved {})
    );

    // Remove a reserved username
    let msg = ExecuteMsg::AdminManageReservedUsernames {
        add_usernames: None,
        remove_usernames: vec!["reserved".to_string()].into(),
    };

    let _res = app
        .execute_contract(admin.clone(), channel_contract_addr.clone(), &msg, &[])
        .unwrap();

    // Query reserved_usernames
    let query_msg = QueryMsg::ReservedUsernames {
        limit: None,
        start_after: None,
    };
    let res: Vec<ReservedUsername> = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();
    assert_eq!(res.len(), 0);

    // Remove usernames while adding
    let msg = ExecuteMsg::AdminManageReservedUsernames {
        add_usernames: Some(vec![ReservedUsername {
            username: "admin".to_string(),
            address: None,
        }]),
        remove_usernames: vec!["admin".to_string()].into(),
    };

    let _res = app
        .execute_contract(admin.clone(), channel_contract_addr.clone(), &msg, &[])
        .unwrap();

    // Query reserved_usernames
    let query_msg = QueryMsg::ReservedUsernames {
        limit: None,
        start_after: None,
    };
    let res: Vec<ReservedUsername> = app
        .wrap()
        .query_wasm_smart(channel_contract_addr.clone(), &query_msg)
        .unwrap();
    assert_eq!(res.len(), 0);
}
