use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
pub type ChannelId = String;
pub type PlaylistId = String;
pub type ChannelsCollectionId = String;

pub const CHANNELS_COLLECTION_ID: Item<ChannelsCollectionId> = Item::new("channels_collection");
pub const CHANNELDETAILS: Map<ChannelId, ChannelDetails> = Map::new("channel_details");
pub const PLAYLISTS: Map<(ChannelId, PlaylistId), Playlist> = Map::new("playlists");
pub const AUTH_DETAILS: Item<AuthDetails> = Item::new("auth_details");
pub const CHANNEL_ID_PAIRS: Map<ChannelId, ChannelDetails> = Map::new("channel_id_pairs");

#[cw_serde]
pub struct Playlist {
    pub playlist_name: String,
    pub assets: Vec<Asset>,
}
impl Playlist {
    pub fn new(playlist_name: String) -> Self {
        Self {
            playlist_name,
            assets: vec![],
        }
    }
}

#[cw_serde]
pub struct Asset {
    pub publish_id: String,
}
#[cw_serde]
pub struct ChannelDetails {
    pub channel_id: String,
}

#[cw_serde]
pub struct AuthDetails {
    pub admin: Addr,
    pub fee_collector: Addr,
}
