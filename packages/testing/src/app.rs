use std::ops::{Deref, DerefMut};

use crate::stargate::StargateKeeper;
use cosmwasm_std::{testing::MockApi, Empty, GovMsg, IbcMsg, IbcQuery, MemoryStorage};
use cw_multi_test::{no_init, App, AppBuilder, BankKeeper, FailingModule, WasmKeeper};

#[allow(clippy::type_complexity)]
pub struct OmniflixApp(
    App<
        BankKeeper,
        MockApi,
        MemoryStorage,
        FailingModule<Empty, Empty, Empty>,
        WasmKeeper<Empty, Empty>,
        FailingModule<Empty, Empty, Empty>,
        FailingModule<Empty, Empty, Empty>,
        FailingModule<IbcMsg, IbcQuery, Empty>,
        FailingModule<Empty, Empty, Empty>,
        StargateKeeper,
    >,
);

impl Deref for OmniflixApp {
    type Target = App<
        BankKeeper,
        MockApi,
        MemoryStorage,
        FailingModule<Empty, Empty, Empty>,
        WasmKeeper<Empty, Empty>,
        FailingModule<Empty, Empty, Empty>,
        FailingModule<Empty, Empty, Empty>,
        FailingModule<IbcMsg, IbcQuery, Empty>,
        FailingModule<Empty, Empty, Empty>,
        StargateKeeper,
    >;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for OmniflixApp {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for OmniflixApp {
    fn default() -> Self {
        Self::new()
    }
}

impl OmniflixApp {
    pub fn new() -> Self {
        let stargate = StargateKeeper {};
        let app_builder = AppBuilder::new();
        let app = app_builder.with_stargate(stargate).build(no_init);
        OmniflixApp(app)
    }
}
