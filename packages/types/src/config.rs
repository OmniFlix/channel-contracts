use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin};

#[cw_serde]
pub struct ChannelConractConfig {
    pub channels_collection_id: String,
    pub channel_creation_fee: Vec<Coin>,
    pub accepted_tip_denoms: Vec<String>,
    pub auth_details: AuthDetails,
}

#[cw_serde]
pub struct AuthDetails {
    pub protocol_admin: Addr,
    pub fee_collector: Addr,
}
