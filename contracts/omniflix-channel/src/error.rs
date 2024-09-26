use asset_manager::error::AssetError;
use asset_manager::error::PlaylistError;
use channel_manager::error::ChannelError;
use cosmwasm_std::{Coin, StdError};
use pauser::PauseError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error(transparent)]
    Pause(#[from] PauseError),

    #[error(transparent)]
    Channel(#[from] ChannelError),

    #[error(transparent)]
    Playlist(#[from] PlaylistError),

    #[error(transparent)]
    Asset(#[from] AssetError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid minter code id")]
    InvalidMinterCodeId {},

    #[error("Invalid Mint Denom")]
    InvalidMintDenom {},

    #[error("Mint denom not allowed")]
    MintDenomNotAllowed {},

    #[error("Missing creation fee")]
    MissingCreationFee {},

    #[error("Missing minter creation fee")]
    MissingMinterCreationFee {},

    #[error("MultiMinter not enabled")]
    MultiMinterNotEnabled {},

    #[error("Channel already exists")]
    ChannelAlreadyExists {},

    #[error("Channel not found")]
    ChannelNotFound {},

    #[error("Payment error")]
    PaymentError {
        expected: Vec<Coin>,
        received: Vec<Coin>,
    },

    #[error("Invalid ONFT data")]
    InvalidOnftData {},

    #[error("Asset to be published not found")]
    AssetNotFound {},

    #[error("Channel ONFT not found")]
    ChannelOnftNotFound {},

    #[error("Query ONFT failed")]
    OnftQueryFailed {},

    #[error("ONFT not found")]
    OnftNotFound {
        collection_id: String,
        onft_id: String,
    },

    #[error("ONFT not owned by the sender")]
    OnftNotOwned {
        collection_id: String,
        onft_id: String,
    },

    #[error("Failed to fetch collection creation fee")]
    CollectionCreationFeeError {},

    #[error("Playlist already exists")]
    PlaylistAlreadyExists {},

    #[error("Playlist does not exist")]
    PlaylistNotFound {},

    #[error("Asset already exists in the playlist")]
    AssetAlreadyExistsInPlaylist {},

    #[error("Asset does not exist in the playlist")]
    AssetNotInPlaylist {},

    #[error("Cannot delete the default playlist")]
    CannotDeleteDefaultPlaylist {},

    #[error("Invalid username, must be between 3 and 32 characters")]
    InvalidUserName {},

    #[error("Invalid description, must be between 3 and 256 characters")]
    InvalidDescription {},

    #[error("Channel ID already exists")]
    ChannelIdAlreadyExists {},

    #[error("Username already taken")]
    UserNameAlreadyTaken {},

    #[error("Channel ID does not exist")]
    ChannelIdNotFound {},

    #[error("Username does not exist")]
    UserNameNotFound {},

    #[error("Invalid channel query")]
    InvalidChannelQuery {},

    #[error("Asset is not visible")]
    AssetNotVisible {},
}

impl From<ContractError> for StdError {
    fn from(err: ContractError) -> StdError {
        StdError::generic_err(err.to_string())
    }
}
