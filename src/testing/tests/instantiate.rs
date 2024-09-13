use crate::{msg::InstantiateMsg, testing::setup::setup};
use cosmwasm_std::Decimal;
use cosmwasm_std::{coin, coins, Addr, BlockInfo, Timestamp, Uint128};
use cw_multi_test::Executor;

#[test]
fn instantiate_channel_contract() {
    // Setup     testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();
    let creator = setup_response.test_accounts.creator.clone();
    let collector = setup_response.test_accounts.collector.clone();

    let instantiate_msg = InstantiateMsg {
        admin: setup_response.test_accounts.admin.clone(),
        channel_creation_fee: vec![],
        fee_collector: setup_response.test_accounts.admin,
        channels_collection_id: "Channels".to_string(),
        channels_collection_name: "Channels".to_string(),
        channels_collection_symbol: "CH".to_string(),
    };

    // Missed Onft collection creation fee. This variable is set to 1000000 uflix
    let _res = app
        .instantiate_contract(
            setup_response.channel_contract_code_id,
            admin.clone(),
            &instantiate_msg,
            &[],
            "Instantiate Channel Contract",
            None,
        )
        .unwrap_err();

    // Happy path
    let res = app
        .instantiate_contract(
            setup_response.channel_contract_code_id,
            admin.clone(),
            &instantiate_msg,
            &[coin(1000000, "uflix")],
            "Instantiate Channel Contract",
            None,
        )
        .unwrap();
}
