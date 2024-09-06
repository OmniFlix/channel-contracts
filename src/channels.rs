use cosmwasm_schema::cw_serde;
use cosmwasm_std::{StdError, StdResult, Storage};

use crate::state::{
    CHANNELDETAILS, CHANNEL_ID_TO_ONFT_ID, CHANNEL_ID_TO_USERNAME, USERNAME_TO_CHANNEL_ID,
};

pub type ChannelId = String;

pub type UserName = String;
pub type ChannelsCollectionId = String;

#[cw_serde]
pub struct ChannelDetails {
    pub channel_id: String,
    pub user_name: String,
    pub onft_id: String,
    pub description: String,
}

// Main contract implementation to manage channels
pub struct ChannelContract<'a> {
    pub storage: &'a mut dyn Storage,
}

impl<'a> ChannelContract<'a> {
    pub fn new(storage: &'a mut dyn Storage) -> Self {
        Self { storage }
    }

    // Function to add a new channel with uniqueness checks
    pub fn add_channel(
        &mut self,
        channel_id: ChannelId,
        user_name: UserName,
        onft_id: String,
        description: String,
    ) -> StdResult<()> {
        // Check if the channel ID already exists
        if CHANNELDETAILS.has(self.storage, channel_id.clone()) {
            return Err(StdError::generic_err("Channel ID already exists"));
        }

        // Check if the username is already mapped to another channel
        if USERNAME_TO_CHANNEL_ID.has(self.storage, user_name.clone()) {
            return Err(StdError::generic_err("Username already taken"));
        }

        if CHANNEL_ID_TO_ONFT_ID.has(self.storage, channel_id.clone()) {
            return Err(StdError::generic_err("Channel ID already exists"));
        }

        // Create and save channel details
        let channel_details = ChannelDetails {
            channel_id: channel_id.clone(),
            user_name: user_name.clone(),
            onft_id,
            description,
        };
        CHANNELDETAILS.save(self.storage, channel_id.clone(), &channel_details)?;

        // Map username to channel ID and channel ID to username
        USERNAME_TO_CHANNEL_ID.save(self.storage, user_name.clone(), &channel_id)?;
        CHANNEL_ID_TO_USERNAME.save(self.storage, channel_id, &user_name)?;

        Ok(())
    }
}
