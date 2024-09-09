use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, StdError, StdResult, Storage};
use cw_storage_plus::Map;

pub const USERNAME_TO_CHANNEL_ID: Map<UserName, ChannelId> = Map::new("username_to_channel_id");

pub const CHANNEL_ID_TO_USERNAME: Map<ChannelId, UserName> = Map::new("channel_id_to_username");
pub const CHANNEL_ID_TO_ONFT_ID: Map<ChannelId, OnftId> = Map::new("channel_id_to_onft_id");
pub const CHANNELDETAILS: Map<ChannelId, ChannelDetails> = Map::new("channel_details");

pub type ChannelId = String;
pub type UserName = String;
pub type OnftId = String;

#[cw_serde]
pub struct ChannelDetails {
    pub channel_id: String,
    pub user_name: String,
    pub onft_id: String,
    pub description: String,
}

impl ChannelDetails {
    pub fn new(
        channel_id: String,
        user_name: String,
        onft_id: String,
        description: String,
    ) -> Self {
        Self {
            channel_id,
            user_name,
            onft_id,
            description,
        }
    }

    pub fn validate_channel_details(&self) -> Result<(), StdError> {
        let user_name = &self.user_name;
        let description = &self.description;

        if user_name.len() < 3 || user_name.len() > 32 {
            return Err(StdError::generic_err(
                "Username must be between 3 and 32 characters",
            ));
        }

        if description.len() < 3 || description.len() > 256 {
            return Err(StdError::generic_err(
                "Description must be between 3 and 256 characters",
            ));
        }
        Ok(())
    }
}

pub struct Channels<'a> {
    pub storage: &'a mut dyn Storage,
}

impl<'a> Channels<'a> {
    pub fn new(storage: &'a mut dyn Storage) -> Self {
        Self { storage }
    }

    // Function to add a new channel with uniqueness checks
    pub fn add_channel(
        &mut self,
        channel_id: ChannelId,
        user_name: UserName,
        onft_id: String,
        channel_details: ChannelDetails,
    ) -> StdResult<()> {
        // Check if the channel ID already exists
        if CHANNELDETAILS.has(self.storage, channel_id.clone()) {
            return Err(StdError::generic_err("Channel ID already exists"));
        }

        // Check if the username is already mapped to another channel
        if USERNAME_TO_CHANNEL_ID.has(self.storage, user_name.clone()) {
            return Err(StdError::generic_err("Username already taken"));
        }

        // Create and save channel details
        CHANNELDETAILS.save(self.storage, channel_id.clone(), &channel_details)?;

        // Map username to channel ID and channel ID to username
        USERNAME_TO_CHANNEL_ID.save(self.storage, user_name.clone(), &channel_id)?;
        CHANNEL_ID_TO_USERNAME.save(self.storage, channel_id, &user_name)?;

        Ok(())
    }

    pub fn get_channel_details(&self, channel_id: ChannelId) -> Result<ChannelDetails, StdError> {
        let channel_details = CHANNELDETAILS.load(self.storage, channel_id)?;
        Ok(channel_details)
    }
    pub fn set_channel_details(
        &mut self,
        channel_id: ChannelId,
        channel_details: ChannelDetails,
    ) -> StdResult<()> {
        // Check if the channel ID exists
        if !CHANNELDETAILS.has(self.storage, channel_id.clone()) {
            return Err(StdError::generic_err("Channel ID does not exist"));
        }
        CHANNELDETAILS.save(self.storage, channel_id, &channel_details)?;

        Ok(())
    }
}

#[cw_serde]
pub struct ChannelOnftData {
    pub channel_id: String,
    pub user_name: String,
}
