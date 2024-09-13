use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::Item;

pub type ChannelsCollectionId = String;

// Define storage items
pub const CONFIG: Item<ChannelConractConfig> = Item::new("channel_registry_params");
pub const AUTH_DETAILS: Item<AuthDetails> = Item::new("auth_details");

#[cw_serde]
pub struct AuthDetails {
    pub admin: Addr,
    pub fee_collector: Addr,
}

#[cw_serde]
pub struct ChannelConractConfig {
    pub channels_collection_name: String,
    pub channels_collection_symbol: String,
    pub channels_collection_id: String,
    pub admin: Addr,
    pub fee_collector: Addr,
    pub channel_creation_fee: Vec<Coin>,
}
