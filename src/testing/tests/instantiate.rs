use crate::msg::QueryMsg;
use crate::state::ChannelConractConfig;
use crate::ContractError;
use crate::{msg::InstantiateMsg, testing::setup::setup};
use cosmwasm_std::coin;
use cw_multi_test::Executor;

#[test]
fn instantiate_channel_contract() {
    // Setup     testing environment
    let setup_response = setup();
    let mut app = setup_response.app;

    // Actors
    let admin = setup_response.test_accounts.admin.clone();

    let instantiate_msg = InstantiateMsg {
        admin: setup_response.test_accounts.admin.clone(),
        channel_creation_fee: vec![],
        fee_collector: setup_response.test_accounts.admin,
        channels_collection_id: "Channels".to_string(),
        channels_collection_name: "Channels".to_string(),
        channels_collection_symbol: "CH".to_string(),
    };

    // Missed Onft collection creation fee. This variable is set to 1000000 uflix
    let res = app
        .instantiate_contract(
            setup_response.channel_contract_code_id,
            admin.clone(),
            &instantiate_msg,
            &[],
            "Instantiate Channel Contract",
            None,
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let typed_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::PaymentError {
            expected: [coin(1000000, "uflix")].to_vec(),
            received: (vec![])
        }
    );

    // Send more than the required fee
    let res = app
        .instantiate_contract(
            setup_response.channel_contract_code_id,
            admin.clone(),
            &instantiate_msg,
            &[coin(1000001, "uflix")],
            "Instantiate Channel Contract",
            None,
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let typed_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(
        typed_err,
        &ContractError::PaymentError {
            expected: [coin(1000000, "uflix")].to_vec(),
            received: [coin(1000001, "uflix")].to_vec()
        }
    );

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

    // Check the contract config
    let config: ChannelConractConfig = app
        .wrap()
        .query_wasm_smart(res, &QueryMsg::Config {})
        .unwrap();
    assert_eq!(config.admin, admin);
    assert_eq!(config.channels_collection_id, "Channels");
    assert_eq!(config.channels_collection_name, "Channels");
    assert_eq!(config.channels_collection_symbol, "CH");
    assert_eq!(config.channel_creation_fee, vec![]);
}
