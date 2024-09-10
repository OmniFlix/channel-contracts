use cosmwasm_schema::cw_serde;
use cosmwasm_std::{StdError, StdResult};
use cw_storage_plus::Map;

pub type ChannelId = String;
pub type PlaylistName = String;
pub type UserName = String;
pub type ChannelsCollectionId = String;

pub const PLAYLISTS: Map<(ChannelId, PlaylistName), Playlist> = Map::new("playlists");
#[cw_serde]
pub struct Playlist {
    pub assets: Vec<Asset>,
}

pub struct Playlists<'a> {
    pub storage: &'a mut dyn cosmwasm_std::Storage,
}

impl<'a> Playlists<'a> {
    pub fn new(storage: &'a mut dyn cosmwasm_std::Storage) -> Self {
        Self { storage }
    }

    pub fn initilize_playlist_for_new_channel(&mut self, channel_id: ChannelId) {
        let playlist = Playlist { assets: vec![] };
        PLAYLISTS
            .save(
                self.storage,
                (channel_id, "My Videos".to_string()),
                &playlist,
            )
            .unwrap();
    }

    pub fn add_asset_to_playlist(
        &mut self,
        channel_id: ChannelId,
        playlist_name: PlaylistName,
        asset: Asset,
    ) -> StdResult<()> {
        let mut playlist = PLAYLISTS
            .load(self.storage, (channel_id.clone(), playlist_name.clone()))
            .map_err(|_| StdError::generic_err("Playlist does not exist"))?;

        // Check if asset already exists in the playlist
        if playlist.assets.contains(&asset) {
            return Err(StdError::generic_err(
                "Asset already exists in the playlist",
            ));
        }
        playlist.assets.push(asset);

        PLAYLISTS.save(self.storage, (channel_id, playlist_name), &playlist)?;
        Ok(())
    }

    pub fn get_playlist(
        &self,
        channel_id: ChannelId,
        playlist_name: PlaylistName,
    ) -> Result<Playlist, StdError> {
        let playlist = PLAYLISTS
            .load(self.storage, (channel_id, playlist_name))
            .or_else(|_| Err(StdError::generic_err("Playlist does not exist")))?;
        Ok(playlist)
    }

    pub fn add_new_playlist(
        &mut self,
        channel_id: ChannelId,
        playlist_name: PlaylistName,
    ) -> StdResult<()> {
        let playlist = Playlist { assets: vec![] };
        // Check if playlist already exists
        if PLAYLISTS.has(self.storage, (channel_id.clone(), playlist_name.clone())) {
            return Err(StdError::generic_err("Playlist already exists"));
        }

        PLAYLISTS
            .save(self.storage, (channel_id, playlist_name), &playlist)
            .unwrap();

        Ok(())
    }
}

#[cw_serde]
pub struct Asset {
    pub publish_id: String,
    pub collection_id: String,
    pub onft_id: String,
}
