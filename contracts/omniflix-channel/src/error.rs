use asset_manager::error::AssetError;
use asset_manager::error::PlaylistError;
use channel_manager::error::ChannelError;
use cosmwasm_std::{Coin, OverflowError, StdError};
use cw_utils::PaymentError;
use pauser::PauseError;
use thiserror::Error;

use crate::string_validation::StringValidationError;

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

    #[error(transparent)]
    Payment(#[from] PaymentError),

    #[error(transparent)]
    StringValidationError(#[from] StringValidationError),

    #[error(transparent)]
    Overflow(#[from] OverflowError),

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

    #[error("Payment error")]
    PaymentError {
        expected: Vec<Coin>,
        received: Vec<Coin>,
    },

    #[error("Invalid ONFT data")]
    InvalidOnftData {},

    #[error("Asset to be published not found")]
    AssetNotFound {},

    #[error("ONFT not found collection_id: {collection_id} onft_id: {onft_id}")]
    OnftNotFound {
        collection_id: String,
        onft_id: String,
    },

    #[error("ONFT not owned collection_id: {collection_id} onft_id: {onft_id}")]
    OnftNotOwned {
        collection_id: String,
        onft_id: String,
    },

    #[error("Failed to fetch collection creation fee")]
    CollectionCreationFeeError {},

    #[error("Username already taken")]
    UserNameAlreadyTaken {},

    #[error("Username does not exist")]
    UserNameNotFound {},

    #[error("Invalid channel query")]
    InvalidChannelQuery {},

    #[error("Asset is not visible")]
    AssetNotVisible {},

    #[error("Invalid user name")]
    InvalidUserName {},

    #[error("Invalid description")]
    InvalidDescription {},

    #[error("Username reserved")]
    UserNameReserved {},

    #[error("Invalid link")]
    InvalidLink {},

    #[error("Invalid channel name")]
    InvalidChannelName {},

    #[error("Invalid tip denom")]
    InvalidTipDenom {},

    #[error("Invalid tip amount")]
    InvalidTipAmount {},
}

impl From<ContractError> for StdError {
    fn from(err: ContractError) -> StdError {
        StdError::generic_err(err.to_string())
    }
}
