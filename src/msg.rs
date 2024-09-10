use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Coin};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Addr,
    pub fee_collector: Addr,
    pub channels_collection_id: String,
    pub channels_collection_name: String,
    pub channels_collection_symbol: String,
    pub channel_creation_fee: Coin,
}

#[cw_serde]
pub enum ExecuteMsg {
    Pause {},
    Unpause {},
    SetPausers {
        pausers: Vec<String>,
    },
    Publish {
        asset_onft_collection_id: String,
        asset_onft_id: String,
        salt: Binary,
        channel_id: String,
        playlist_id: Option<String>,
    },
    CreatePlaylist {
        playlist_id: String,
        channel_id: String,
    },
    RegisterChannel {
        salt: Binary,
        user_name: String,
        description: String,
    },
    SetChannelDetails {
        channel_id: String,
        description: String,
    },
}

#[cw_serde]
pub enum QueryMsg {
    IsPaused {},
    Pausers {},
    ChannelDetails {
        channel_id: String,
    },
    ChannelId {
        user_name: String,
    },
    Playlist {
        channel_id: String,
        playlist_id: String,
    },
    Channels {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    Playlists {
        channel_id: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
}
