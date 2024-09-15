use crate::ContractError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Order, StdResult, Storage};
use cw_storage_plus::{Bound, Map};

pub type ChannelId = String;
pub type PlaylistName = String;

#[cw_serde]
pub struct Playlist {
    pub playlist_name: String,
    pub assets: Vec<Asset>,
}

#[cw_serde]
pub struct Asset {
    pub publish_id: String,
    pub collection_id: String,
    pub onft_id: String,
}

const PLAYLISTS_STORAGE_KEY: &str = "playlists";
const DEFAULT_PLAYLIST_NAME: &str = "My Videos";

pub struct PlaylistsManager<'a> {
    pub playlists: Map<'a, (ChannelId, PlaylistName), Playlist>,
}

impl<'a> PlaylistsManager<'a> {
    pub const fn new() -> Self {
        PlaylistsManager {
            playlists: Map::new(PLAYLISTS_STORAGE_KEY),
        }
    }

    // Initialize a default playlist for a new channel
    pub fn initialize_playlist_for_new_channel(
        &self,
        store: &mut dyn Storage,
        channel_id: ChannelId,
    ) -> StdResult<()> {
        let playlist = Playlist {
            playlist_name: DEFAULT_PLAYLIST_NAME.to_string(),
            assets: vec![],
        };
        self.playlists.save(
            store,
            (channel_id, DEFAULT_PLAYLIST_NAME.to_string()),
            &playlist,
        )?;
        Ok(())
    }

    // Add a new playlist to a channel
    pub fn add_new_playlist(
        &self,
        store: &mut dyn Storage,
        channel_id: ChannelId,
        playlist_name: PlaylistName,
    ) -> Result<(), ContractError> {
        let playlist = Playlist {
            assets: vec![],
            playlist_name: playlist_name.clone(),
        };

        if self
            .playlists
            .has(store, (channel_id.clone(), playlist_name.clone()))
        {
            return Err(ContractError::PlaylistAlreadyExists {});
        }

        self.playlists
            .save(store, (channel_id, playlist_name), &playlist)?;

        Ok(())
    }

    // Add an asset to a specific playlist
    pub fn add_asset_to_playlist(
        &self,
        store: &mut dyn Storage,
        channel_id: ChannelId,
        playlist_name: PlaylistName,
        asset: Asset,
    ) -> Result<(), ContractError> {
        let mut playlist = self
            .playlists
            .load(store, (channel_id.clone(), playlist_name.clone()))
            .map_err(|_| ContractError::PlaylistNotFound {})?;

        if playlist.assets.contains(&asset) {
            return Err(ContractError::AssetAlreadyExistsInPlaylist {});
        }

        playlist.assets.push(asset);

        self.playlists
            .save(store, (channel_id, playlist_name), &playlist)?;

        Ok(())
    }

    // Remove an asset from a playlist
    pub fn remove_asset_from_playlist(
        &self,
        store: &mut dyn Storage,
        channel_id: ChannelId,
        playlist_name: PlaylistName,
        asset: Asset,
    ) -> Result<(), ContractError> {
        let mut playlist = self
            .playlists
            .load(store, (channel_id.clone(), playlist_name.clone()))
            .map_err(|_| ContractError::PlaylistNotFound {})?;

        if !playlist.assets.contains(&asset) {
            return Err(ContractError::AssetNotInPlaylist {});
        }

        playlist.assets.retain(|a| a != &asset);

        self.playlists
            .save(store, (channel_id, playlist_name), &playlist)?;

        Ok(())
    }

    // Get a specific playlist
    pub fn get_playlist(
        &self,
        store: &dyn Storage,
        channel_id: ChannelId,
        playlist_name: PlaylistName,
    ) -> Result<Playlist, ContractError> {
        self.playlists
            .load(store, (channel_id, playlist_name))
            .map_err(|_| ContractError::PlaylistNotFound {})
    }

    // Get all playlists for a channel (with pagination support)
    pub fn get_all_playlists(
        &self,
        store: &dyn Storage,
        channel_id: ChannelId,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<Vec<Playlist>> {
        let limit = limit.unwrap_or(25) as usize;
        let start = start_after.map(Bound::exclusive);

        let playlists = self
            .playlists
            .prefix(channel_id)
            .range(store, start, None, Order::Ascending)
            .take(limit)
            .map(|item| item.map(|(_, playlist)| playlist))
            .collect::<StdResult<Vec<Playlist>>>()?;

        Ok(playlists)
    }

    // Remove a playlist (cannot remove the default playlist)
    pub fn remove_playlist(
        &self,
        store: &mut dyn Storage,
        channel_id: ChannelId,
        playlist_name: PlaylistName,
    ) -> Result<(), ContractError> {
        if playlist_name == DEFAULT_PLAYLIST_NAME {
            return Err(ContractError::CannotDeleteDefaultPlaylist {});
        }

        self.playlists.remove(store, (channel_id, playlist_name));
        Ok(())
    }
}
