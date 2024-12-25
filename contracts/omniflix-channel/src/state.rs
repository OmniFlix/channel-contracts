use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use omniflix_channel_types::config::ChannelConractConfig;

pub type ChannelsCollectionId = String;

// Define storage items
pub const CONFIG: Item<ChannelConractConfig> = Item::new("channel_registry_params");
pub const AUTH_DETAILS: Item<AuthDetails> = Item::new("auth_details");

#[cw_serde]
pub struct AuthDetails {
    pub admin: Addr,
    pub fee_collector: Addr,
}
