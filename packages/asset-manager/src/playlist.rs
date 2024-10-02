use cosmwasm_std::{Order, StdResult, Storage};
use cw_storage_plus::{Bound, Map};

use crate::{
    assets::Assets,
    error::PlaylistError,
    types::{Asset, Playlist},
};
type ChannelId = String;
type PlaylistName = String;

const PLAYLISTS_STORAGE_KEY: &str = "playlists";

pub struct PlaylistsManager<'a> {
    pub playlists: Map<'a, (ChannelId, PlaylistName), Playlist>,
}

impl<'a> PlaylistsManager<'a> {
    pub const fn new() -> Self {
        PlaylistsManager {
            playlists: Map::new(PLAYLISTS_STORAGE_KEY),
        }
    }
    // Add a new playlist to a channel
    pub fn add_new_playlist(
        &self,
        store: &mut dyn Storage,
        channel_id: ChannelId,
        playlist_name: PlaylistName,
    ) -> Result<(), PlaylistError> {
        let playlist = Playlist {
            assets: vec![],
            playlist_name: playlist_name.clone(),
        };

        if self
            .playlists
            .has(store, (channel_id.clone(), playlist_name.clone()))
        {
            return Err(PlaylistError::PlaylistAlreadyExists {});
        }

        self.playlists
            .save(store, (channel_id, playlist_name), &playlist)
            .map_err(|_| PlaylistError::PlaylistAlreadyExists {})?;

        Ok(())
    }

    // Add an asset to a specific playlist
    pub fn add_asset_to_playlist(
        &self,
        store: &mut dyn Storage,
        channel_id: ChannelId,
        playlist_name: PlaylistName,
        asset: Asset,
    ) -> Result<(), PlaylistError> {
        let mut playlist = self
            .playlists
            .load(store, (channel_id.clone(), playlist_name.clone()))
            .map_err(|_| PlaylistError::PlaylistNotFound {})?;

        if playlist.assets.contains(&asset) {
            return Err(PlaylistError::AssetAlreadyExistsInPlaylist {});
        }

        playlist.assets.push(asset);

        self.playlists
            .save(store, (channel_id, playlist_name), &playlist)
            .map_err(|_| PlaylistError::SavePlaylistError {})?;

        Ok(())
    }

    // Remove an asset from a playlist
    pub fn remove_asset_from_playlist(
        &self,
        store: &mut dyn Storage,
        channel_id: ChannelId,
        playlist_name: PlaylistName,
        publish_id: String,
    ) -> Result<(), PlaylistError> {
        let mut playlist = self
            .playlists
            .load(store, (channel_id.clone(), playlist_name.clone()))
            .map_err(|_| PlaylistError::PlaylistNotFound {})?;

        let asset_index = playlist
            .assets
            .iter()
            .position(|asset| asset.publish_id == publish_id)
            .ok_or(PlaylistError::AssetNotInPlaylist {})?;

        playlist.assets.remove(asset_index);

        self.playlists
            .save(store, (channel_id, playlist_name), &playlist)
            .map_err(|_| PlaylistError::SavePlaylistError {})?;

        Ok(())
    }

    // Get a specific playlist
    pub fn get_playlist(
        &self,
        store: &dyn Storage,
        channel_id: ChannelId,
        playlist_name: PlaylistName,
    ) -> Result<Playlist, PlaylistError> {
        self.playlists
            .load(store, (channel_id, playlist_name))
            .map_err(|_| PlaylistError::PlaylistNotFound {})
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

    // Delete a playlist
    pub fn delete_playlist(
        &self,
        store: &mut dyn Storage,
        channel_id: ChannelId,
        playlist_name: PlaylistName,
    ) -> Result<(), PlaylistError> {
        if self
            .playlists
            .load(store, (channel_id.clone(), playlist_name.clone()))
            .is_err()
        {
            return Err(PlaylistError::PlaylistNotFound {});
        }

        self.playlists.remove(store, (channel_id, playlist_name));
        Ok(())
    }

    // Refresh the playlist
    // We iterate through all the playlists and remove any assets that are not found in the asset manager or are not visible
    pub fn refresh_playlist(
        &self,
        store: &mut dyn Storage,
        channel_id: ChannelId,
        playlist_name: PlaylistName,
    ) -> Result<Vec<Asset>, PlaylistError> {
        // Load the playlist
        let mut playlist = self
            .playlists
            .load(store, (channel_id.clone(), playlist_name.clone()))
            .map_err(|_| PlaylistError::PlaylistNotFound {})?;

        let assets_in_playlist = playlist.assets.clone();
        let asset_manager = Assets::new();

        // Filter and collect visible assets
        let new_assets: Vec<Asset> = assets_in_playlist
            .iter()
            .filter_map(|asset| {
                asset_manager
                    .get_asset(store, channel_id.clone(), asset.publish_id.clone())
                    .ok()
                    .filter(|asset_details| asset_details.is_visible)
                    .map(|_| asset.clone()) // If asset is visible, clone it for new playlist
            })
            .collect();

        // Update the playlist with filtered assets
        playlist.assets = new_assets.clone();

        // Save the updated playlist back to storage
        self.playlists
            .save(store, (channel_id, playlist_name), &playlist)
            .map_err(|_| PlaylistError::SavePlaylistError {})?;

        // Return the removed assets (those that were not added to the new_assets)
        Ok(assets_in_playlist
            .into_iter()
            .filter(|asset| !new_assets.contains(asset))
            .collect())
    }

    // Delete all playlists for a channel
    pub fn delete_playlists_by_channel_id(&self, store: &mut dyn Storage, channel_id: ChannelId) {
        self.playlists.prefix(channel_id).clear(store, None)
    }
}

// Test delete playlists by channel id
#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::MockStorage;

    #[test]
    fn test_delete_playlists_by_channel_id() {
        let mut storage = MockStorage::new();
        let playlists_manager = PlaylistsManager::new();

        let channel_id = "channel_id".to_string();
        let playlist_name = "playlist_name".to_string();

        // Add 100 playlists
        for i in 0..24 {
            let playlist_name = format!("playlist_{}", i);
            playlists_manager
                .add_new_playlist(&mut storage, channel_id.clone(), playlist_name)
                .unwrap();
        }

        // Check if the playlists are added
        let all_playlists = playlists_manager
            .get_all_playlists(&storage, channel_id.clone(), None, None)
            .unwrap();
        assert_eq!(all_playlists.len(), 24);

        // Delete all playlists for the channel
        playlists_manager.delete_playlists_by_channel_id(&mut storage, channel_id.clone());

        let all_playlists = playlists_manager
            .get_all_playlists(&storage, channel_id.clone(), None, None)
            .unwrap();
        assert_eq!(all_playlists.len(), 0);

        // Check if the playlist is deleted
        let playlist =
            playlists_manager.get_playlist(&storage, channel_id.clone(), playlist_name.clone());
        assert!(playlist.is_err());
    }
}
