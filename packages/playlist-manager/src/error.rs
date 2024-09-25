use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Order, StdResult, Storage};
use cw_storage_plus::{Bound, Map};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum PlaylistError {
    #[error("Playlist not found")]
    PlaylistNotFound {},

    #[error("Playlist already exists")]
    PlaylistAlreadyExists {},

    #[error("Asset already exists in playlist")]
    AssetAlreadyExistsInPlaylist {},

    #[error("Asset not in playlist")]
    AssetNotInPlaylist {},

    #[error("Cannot delete default playlist")]
    CannotDeleteDefaultPlaylist {},

    #[error("Error saving playlist")]
    SavePlaylistError {},
}

#[derive(Error, Debug, PartialEq)]
pub enum AssetError {
    #[error("Asset not found")]
    AssetNotFound {},

    #[error("Asset already exists")]
    AssetAlreadyExists {},

    #[error("Error saving asset")]
    SaveAssetError {},
}
