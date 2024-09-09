use cosmwasm_schema::cw_serde;
use cosmwasm_std::{StdError, StdResult, Storage};
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

    pub fn get_channel_details(&self, channel_id: ChannelId) -> Result<ChannelDetails, StdError> {
        let channel_details = CHANNELDETAILS.load(self.storage, channel_id)?;
        Ok(channel_details)
    }

    // Check if the channel ID exists
    pub fn channel_exists(&self, channel_id: ChannelId) -> bool {
        CHANNELDETAILS.has(self.storage, channel_id)
    }

    pub fn set_channel_details(
        &mut self,
        channel_id: ChannelId,
        description: String,
    ) -> StdResult<()> {
        // Check if the channel ID exists
        if !CHANNELDETAILS.has(self.storage, channel_id.clone()) {
            return Err(StdError::generic_err("Channel ID does not exist"));
        }

        // Update the channel details
        let mut channel_details = CHANNELDETAILS.load(self.storage, channel_id.clone())?;
        channel_details.description = description;
        CHANNELDETAILS.save(self.storage, channel_id, &channel_details)?;

        Ok(())
    }
}

#[cw_serde]
pub struct ChannelOnftData {
    pub channel_id: String,
    pub user_name: String,
}
