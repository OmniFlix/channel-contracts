use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin};

#[cw_serde]
pub struct ChannelConractConfig {
    pub channels_collection_name: String,
    pub channels_collection_symbol: String,
    pub channels_collection_id: String,
    pub admin: Addr,
    pub fee_collector: Addr,
    pub channel_creation_fee: Vec<Coin>,
}
