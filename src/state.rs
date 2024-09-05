use cosmwasm_std::{Addr, HexBinary};
use cw_storage_plus::Item;

pub const PARTICIPANT_COUNT: Item<u32> = Item::new("participant_count");
pub const NOIS_PROXY: Item<Addr> = Item::new("nois_proxy");
pub const ADMIN: Item<Addr> = Item::new("admin");
pub const WINNERS: Item<Vec<u32>> = Item::new("winners");
pub const TEST_WINNERS: Item<Vec<u32>> = Item::new("test_winners");
pub const TEST_RANDOMNESS: Item<HexBinary> = Item::new("test_randomness");
pub const FINAL_RANDOMNESS: Item<HexBinary> = Item::new("final_randomness");
