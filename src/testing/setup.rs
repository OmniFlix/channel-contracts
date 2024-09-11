use cosmwasm_std::{coins, Addr, BlockInfo, Timestamp};
use cw_multi_test::ContractWrapper;

use crate::testing::app::OmniflixApp;

use super::utils::mint_to_address;

pub fn setup() -> SetupResponse {
    let mut app = OmniflixApp::new();
    let admin = Addr::unchecked("admin");
    let creator = Addr::unchecked("creator");
    let collector = Addr::unchecked("collector");

    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_nanos(1_000),
    });
    mint_to_address(&mut app, admin.to_string(), coins(1000000000, "uflix"));
    mint_to_address(&mut app, creator.to_string(), coins(1000000000, "uflix"));
    mint_to_address(&mut app, collector.to_string(), coins(1000000000, "uflix"));
    mint_to_address(
        &mut app,
        collector.to_string(),
        coins(1000000000000, "diffirent_denom"),
    );
    mint_to_address(
        &mut app,
        collector.to_string(),
        coins(1000000000000, "incorrect_denom"),
    );
    mint_to_address(
        &mut app,
        creator.to_string(),
        coins(1000000000000, "incorrect_denom"),
    );
    mint_to_address(
        &mut app,
        creator.to_string(),
        coins(1000000000000, "diffirent_denom"),
    );

    let channel_contract = Box::new(ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    ));

    let minter_code_id = app.store_code(minter_contract);

    SetupResponse {
        app,
        test_accounts: TestAccounts {
            admin,
            creator,
            collector,
        },
        minter_factory_code_id,
        minter_code_id,
        round_whitelist_factory_code_id,
        round_whitelist_code_id,
        open_edition_minter_factory_code_id,
        open_edition_minter_code_id,
        multi_mint_open_edition_minter_code_id,
    }
}

pub struct SetupResponse {
    pub app: OmniflixApp,
    pub test_accounts: TestAccounts,
    pub minter_factory_code_id: u64,
    pub minter_code_id: u64,
    pub round_whitelist_factory_code_id: u64,
    pub round_whitelist_code_id: u64,
    pub open_edition_minter_factory_code_id: u64,
    pub open_edition_minter_code_id: u64,
    pub multi_mint_open_edition_minter_code_id: u64,
}
pub struct TestAccounts {
    pub admin: Addr,
    pub creator: Addr,
    pub collector: Addr,
}
