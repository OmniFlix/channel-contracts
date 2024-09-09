use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, StdError, StdResult, Storage};
use cw_storage_plus::{Item, Map};

use crate::{channels::ChannelDetails, playlist::Playlist};

pub type ChannelId = String;
pub type PlaylistId = String;
pub type UserName = String;
pub type OnftId = String;
pub type ChannelsCollectionId = String;

// Define storage items
pub const CHANNELS_COLLECTION_ID: Item<ChannelsCollectionId> = Item::new("channels_collection");

pub const CONFIG: Item<ChannelConractConfig> = Item::new("channel_registry_params");
pub const AUTH_DETAILS: Item<AuthDetails> = Item::new("auth_details");

pub const USERNAME_TO_CHANNEL_ID: Map<UserName, ChannelId> = Map::new("username_to_channel_id");

pub const CHANNEL_ID_TO_USERNAME: Map<ChannelId, UserName> = Map::new("channel_id_to_username");
pub const CHANNEL_ID_TO_ONFT_ID: Map<ChannelId, OnftId> = Map::new("channel_id_to_onft_id");
pub const CHANNELDETAILS: Map<ChannelId, ChannelDetails> = Map::new("channel_details");
pub const PLAYLISTS: Map<(ChannelId, PlaylistId), Playlist> = Map::new("playlists");

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
    pub channel_creation_fee: Coin,
}
