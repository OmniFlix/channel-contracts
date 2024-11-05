use cosmwasm_std::{coin, Addr, BlockInfo, Timestamp};
use cw_multi_test::{ContractWrapper, MockApiBech32};

use super::utils::mint_to_address;
use testing::app::OmniflixApp;

pub fn setup() -> SetupResponse {
    let api = MockApiBech32::new("cosmwasm");
    let mut app = OmniflixApp::new();

    let admin = api.addr_make("admin");
    let creator = api.addr_make("creator");
    let creator2 = api.addr_make("creator2");
    let collector = api.addr_make("collector");

    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_nanos(1_000),
    });

    // Mint multiple denominations at once for each address
    mint_to_address(
        &mut app,
        creator.to_string(),
        vec![
            coin(1_000_000_000, "uflix"),
            coin(1_000_000_000_000, "diffirent_denom"),
            coin(1_000_000_000_000, "incorrect_denom"),
        ],
    );
    mint_to_address(
        &mut app,
        creator2.to_string(),
        vec![
            coin(1_000_000_000, "uflix"),
            coin(1_000_000_000_000, "diffirent_denom"),
            coin(1_000_000_000_000, "incorrect_denom"),
        ],
    );
    mint_to_address(
        &mut app,
        collector.to_string(),
        vec![
            coin(1_000_000_000, "uflix"),
            coin(1_000_000_000_000, "diffirent_denom"),
            coin(1_000_000_000_000, "incorrect_denom"),
        ],
    );
    mint_to_address(
        &mut app,
        admin.to_string(),
        vec![coin(1_000_000_000, "uflix")],
    );

    let channel_contract = Box::new(ContractWrapper::new(
        omniflix_channel::contract::execute,
        omniflix_channel::contract::instantiate,
        omniflix_channel::contract::query,
    ));

    let channel_contract_code_id = app.store_code(channel_contract);

    SetupResponse {
        app,
        test_accounts: TestAccounts {
            admin,
            creator,
            creator2,
            collector,
        },
        channel_contract_code_id,
    }
}

pub struct SetupResponse {
    pub app: OmniflixApp,
    pub test_accounts: TestAccounts,
    pub channel_contract_code_id: u64,
}

pub struct TestAccounts {
    pub admin: Addr,
    pub creator: Addr,
    pub creator2: Addr,
    pub collector: Addr,
}
