use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, StdError, StdResult, Storage};
use cw_storage_plus::{Item, Map};

pub type ChannelId = String;
pub type PlaylistId = String;
pub type UserName = String;
pub type ChannelsCollectionId = String;

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

    // Add an asset to the playlist
    pub fn add_asset(&mut self, asset: Asset) {
        self.assets.push(asset);
    }
}

#[cw_serde]
pub struct Asset {
    pub publish_id: String,
}
