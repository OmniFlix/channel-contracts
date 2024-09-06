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
        onft_collection_id: String,
        onft_id: String,
        salt: Binary,
        channel_id: String,
        playlist_id: Option<String>,
    },
    CreatePlaylist {
        playlist_id: String,
        channel_id: String,
    },
    RegisterChannel {
        channel_id: String,
        salt: Option<Binary>,
    },
}

#[cw_serde]
pub enum QueryMsg {
    IsPaused {},
    Pausers {},
}
