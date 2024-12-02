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

    #[error("Ipfs link cannot be empty")]
    IpfsLinkCannotBeEmpty {},

    #[error("Name cannot be empty")]
    NameCannotBeEmpty {},

    #[error("Name cannot be longer than 256 characters")]
    NameTooLong {},

    #[error("Description cannot be empty")]
    DescriptionCannotBeEmpty {},

    #[error("Description cannot be longer than 512 characters")]
    DescriptionTooLong {},

    #[error("Ipfs link cannot be longer than 256 characters")]
    IpfsLinkTooLong {},

    #[error("Collection ID cannot be empty")]
    CollectionIdCannotBeEmpty {},

    #[error("Onft ID cannot be empty")]
    OnftIdCannotBeEmpty {},
}
