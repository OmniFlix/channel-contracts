use cosmwasm_std::{Coin, StdError};
use thiserror::Error;

use crate::pauser::PauseError;
#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error(transparent)]
    Pause(#[from] PauseError),

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

    #[error("Channel Onft not found")]
    ChannelOnftNotFound {},
}
impl From<ContractError> for StdError {
    fn from(err: ContractError) -> StdError {
        StdError::generic_err(err.to_string())
    }
}
