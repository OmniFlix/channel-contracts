use cosmwasm_std::{Order, StdResult, Storage};
use cw_storage_plus::{Bound, Map};

use omniflix_channel_types::asset::{AssetKey, Playlist};

use crate::assets::Assets;
use crate::error::PlaylistError;

type ChannelId = String;
type PlaylistName = String;

const PLAYLISTS_STORAGE_KEY: &str = "playlists";

pub struct PlaylistsManager {
    pub playlists: Map<(ChannelId, PlaylistName), Playlist>,
}

impl PlaylistsManager {
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
        if self
            .playlists
            .has(store, (channel_id.clone(), playlist_name.clone()))
        {
            return Err(PlaylistError::PlaylistAlreadyExists {});
        }

        let playlist = Playlist {
            assets: Vec::new(),
            playlist_name: playlist_name.clone(),
        };

        self.playlists
            .save(store, (channel_id, playlist_name), &playlist)
            .map_err(|_| PlaylistError::SavePlaylistError {})?;

        Ok(())
    }

    // Add an asset to a specific playlist
    pub fn add_asset_to_playlist(
        &self,
        store: &mut dyn Storage,
        channel_id: ChannelId,
        playlist_name: PlaylistName,
        asset_key: AssetKey,
    ) -> Result<(), PlaylistError> {
        let mut playlist = self
            .playlists
            .load(store, (channel_id.clone(), playlist_name.clone()))
            .map_err(|_| PlaylistError::PlaylistNotFound {})?;

        if playlist.assets.contains(&asset_key) {
            return Err(PlaylistError::AssetAlreadyExistsInPlaylist {});
        }

        playlist.assets.push(asset_key);

        self.playlists
            .save(store, (channel_id, playlist_name), &playlist)
            .map_err(|_| PlaylistError::SavePlaylistError {})?;

        Ok(())
    }

    // Remove an asset from a playlist
    pub fn remove_assets_from_playlist(
        &self,
        store: &mut dyn Storage,
        channel_id: ChannelId,
        playlist_name: PlaylistName,
        asset_keys: Vec<AssetKey>,
    ) -> Result<(), PlaylistError> {
        let mut playlist = self
            .playlists
            .load(store, (channel_id.clone(), playlist_name.clone()))
            .map_err(|_| PlaylistError::PlaylistNotFound {})?;

        for asset_key in asset_keys.iter() {
            if let Some(index) = playlist.assets.iter().position(|x| x == asset_key) {
                playlist.assets.remove(index);
            } else {
                // Asset not found in the playlist
                return Err(PlaylistError::AssetNotInPlaylist {});
            }
        }

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

        self.playlists
            .prefix(channel_id)
            .range(store, start, None, Order::Ascending)
            .take(limit)
            .map(|item| item.map(|(_, playlist)| playlist))
            .collect()
    }

    // Delete a playlist
    pub fn delete_playlist(
        &self,
        store: &mut dyn Storage,
        channel_id: ChannelId,
        playlist_name: PlaylistName,
    ) -> Result<(), PlaylistError> {
        if !self
            .playlists
            .has(store, (channel_id.clone(), playlist_name.clone()))
        {
            return Err(PlaylistError::PlaylistNotFound {});
        }

        self.playlists.remove(store, (channel_id, playlist_name));
        Ok(())
    }

    // Refresh the playlist
    pub fn refresh_playlist(
        &self,
        store: &mut dyn Storage,
        channel_id: ChannelId,
        playlist_name: PlaylistName,
    ) -> Result<Vec<AssetKey>, PlaylistError> {
        let mut playlist = self
            .playlists
            .load(store, (channel_id.clone(), playlist_name.clone()))
            .map_err(|_| PlaylistError::PlaylistNotFound {})?;

        let asset_manager = Assets::new();
        let mut removed_asset_keys = Vec::with_capacity(playlist.assets.len());

        playlist.assets.retain(|asset_key| {
            if let Ok(asset) = asset_manager.get_asset(store, asset_key.clone()) {
                if asset.is_visible {
                    return true; // Keep visible asset
                }
            }
            removed_asset_keys.push(asset_key.clone()); // Add to removed if not found or not visible
            false // Remove asset
        });

        self.playlists
            .save(store, (channel_id, playlist_name), &playlist)
            .map_err(|_| PlaylistError::SavePlaylistError {})?;

        Ok(removed_asset_keys)
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

    #[test]
    fn test_add_new_playlist() {
        let mut storage = MockStorage::new();
        let playlists_manager = PlaylistsManager::new();

        let channel_id = "channel_id".to_string();
        let playlist_name = "playlist_name".to_string();

        // Add a new playlist
        playlists_manager
            .add_new_playlist(&mut storage, channel_id.clone(), playlist_name.clone())
            .unwrap();

        // Check if the playlist is added
        let playlist =
            playlists_manager.get_playlist(&storage, channel_id.clone(), playlist_name.clone());
        assert!(playlist.is_ok());
    }

    #[test]
    fn test_add_asset_to_playlist() {
        let mut storage = MockStorage::new();
        let playlists_manager = PlaylistsManager::new();

        let channel_id = "channel_id".to_string();
        let playlist_name = "playlist_name".to_string();
        let asset_key: AssetKey = ("asset_channel_id".to_string(), "publish_id".to_string());

        // Add a new playlist
        playlists_manager
            .add_new_playlist(&mut storage, channel_id.clone(), playlist_name.clone())
            .unwrap();

        // Add an asset to the playlist
        playlists_manager
            .add_asset_to_playlist(
                &mut storage,
                channel_id.clone(),
                playlist_name.clone(),
                asset_key.clone(),
            )
            .unwrap();

        // Check if the asset is added to the playlist
        let playlist = playlists_manager
            .get_playlist(&storage, channel_id.clone(), playlist_name.clone())
            .unwrap();
        assert_eq!(playlist.assets.len(), 1);
        assert_eq!(playlist.assets[0], asset_key);
    }

    #[test]
    fn test_remove_assets_from_playlist() {
        let mut storage = MockStorage::new();
        let playlists_manager = PlaylistsManager::new();

        let channel_id = "channel_id".to_string();
        let playlist_name = "playlist_name".to_string();
        let asset_key: AssetKey = ("asset_channel_id".to_string(), "publish_id".to_string());

        // Add a new playlist
        playlists_manager
            .add_new_playlist(&mut storage, channel_id.clone(), playlist_name.clone())
            .unwrap();

        // Add an asset to the playlist
        playlists_manager
            .add_asset_to_playlist(
                &mut storage,
                channel_id.clone(),
                playlist_name.clone(),
                asset_key.clone(),
            )
            .unwrap();

        // Remove the asset from the playlist
        playlists_manager
            .remove_assets_from_playlist(
                &mut storage,
                channel_id.clone(),
                playlist_name.clone(),
                vec![asset_key.clone()],
            )
            .unwrap();

        // Check if the asset is removed from the playlist
        let playlist = playlists_manager
            .get_playlist(&storage, channel_id.clone(), playlist_name.clone())
            .unwrap();
        assert_eq!(playlist.assets.len(), 0);
    }

    #[test]
    fn test_get_playlist() {
        let mut storage = MockStorage::new();
        let playlists_manager = PlaylistsManager::new();

        let channel_id = "channel_id".to_string();
        let playlist_name = "playlist_name".to_string();

        // Add a new playlist
        playlists_manager
            .add_new_playlist(&mut storage, channel_id.clone(), playlist_name.clone())
            .unwrap();

        // Get the playlist
        let playlist = playlists_manager
            .get_playlist(&storage, channel_id.clone(), playlist_name.clone())
            .unwrap();
        assert_eq!(playlist.playlist_name, playlist_name);
    }

    #[test]
    fn test_get_all_playlists_with_start_after() {
        let mut storage = MockStorage::new();
        let playlists_manager = PlaylistsManager::new();

        let channel_id = "channel_id".to_string();

        // Add 100 playlists
        for i in 0..100 {
            let playlist_name = format!("playlist_{}", i);
            playlists_manager
                .add_new_playlist(&mut storage, channel_id.clone(), playlist_name)
                .unwrap();
        }

        // Get 25 playlists starting after the 25th playlist
        let all_playlists = playlists_manager
            .get_all_playlists(
                &storage,
                channel_id.clone(),
                Some("playlist_24".to_string()),
                Some(25),
            )
            .unwrap();
        assert_eq!(all_playlists.len(), 25);
        assert_eq!(all_playlists[0].playlist_name, "playlist_25");
    }

    #[test]
    fn test_delete_playlist() {
        let mut storage = MockStorage::new();
        let playlists_manager = PlaylistsManager::new();

        let channel_id = "channel_id".to_string();
        let playlist_name = "playlist_name".to_string();

        // Add a new playlist
        playlists_manager
            .add_new_playlist(&mut storage, channel_id.clone(), playlist_name.clone())
            .unwrap();

        // Delete the playlist
        playlists_manager
            .delete_playlist(&mut storage, channel_id.clone(), playlist_name.clone())
            .unwrap();

        // Check if the playlist is deleted
        let playlist =
            playlists_manager.get_playlist(&storage, channel_id.clone(), playlist_name.clone());
        assert!(playlist.is_err());
    }
}
