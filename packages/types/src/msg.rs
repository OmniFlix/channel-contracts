use asset_manager::types::{Playlist, Visibility};
use channel_manager::types::ChannelDetails;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, Coin};

use crate::config::ChannelConractConfig;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Addr,
    pub fee_collector: Addr,
    pub channels_collection_id: String,
    pub channels_collection_name: String,
    pub channels_collection_symbol: String,
    pub channel_creation_fee: Vec<Coin>,
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
        playlist_name: Option<String>,
        visibility: Visibility,
    },
    CreatePlaylist {
        playlist_name: String,
        channel_id: String,
    },
    RemovePlaylist {
        playlist_name: String,
        channel_id: String,
    },
    RemoveAsset {
        publish_id: String,
        channel_id: String,
        playlist_name: String,
    },
    CreateChannel {
        salt: Binary,
        user_name: String,
        description: String,
        collabarators: Option<Vec<String>>,
    },
    SetChannelDetails {
        channel_id: String,
        description: String,
    },
    SetConfig {
        channel_creation_fee: Option<Vec<Coin>>,
        admin: Option<String>,
        fee_collector: Option<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(bool)]
    IsPaused {},
    #[returns(Vec<String>)]
    Pausers {},
    #[returns(ChannelDetails)]
    ChannelDetails {
        channel_id: Option<String>,
        user_name: Option<String>,
    },
    #[returns(Vec<ChannelDetails>)]
    Channels {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(String)]
    ChannelId { user_name: String },
    #[returns(Playlist)]
    Playlist {
        channel_id: String,
        playlist_name: String,
    },
    #[returns(Vec<Playlist>)]
    Playlists {
        channel_id: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(ChannelConractConfig)]
    Config {},
}
