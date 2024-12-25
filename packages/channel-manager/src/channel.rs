use cosmwasm_std::{Addr, Order, StdResult, Storage};
use cw_storage_plus::{Bound, Map};

use crate::{
    error::ChannelError,
    types::{ChannelDetails, ChannelId, ChannelMetadata, UserName},
};

const CHANNEL_DETAILS_STORAGE_KEY: &str = "channel_details";
const CHANNEL_METADATA_STORAGE_KEY: &str = "channel_metadata";
const USERNAME_TO_CHANNEL_ID_STORAGE_KEY: &str = "username_to_channel_id";
const CHANNEL_ID_TO_USERNAME_STORAGE_KEY: &str = "channel_id_to_username";
const RESERVED_USERNAMES_STORAGE_KEY: &str = "reserved_usernames";

pub struct ChannelsManager {
    pub channel_details: Map<ChannelId, ChannelDetails>,
    pub channel_metadata: Map<ChannelId, ChannelMetadata>,
    pub username_to_channel_id: Map<UserName, ChannelId>,
    pub channel_id_to_username: Map<ChannelId, UserName>,
    pub reserved_usernames: Map<UserName, Addr>,
}

impl ChannelsManager {
    pub const fn new() -> Self {
        ChannelsManager {
            channel_details: Map::new(CHANNEL_DETAILS_STORAGE_KEY),
            username_to_channel_id: Map::new(USERNAME_TO_CHANNEL_ID_STORAGE_KEY),
            channel_id_to_username: Map::new(CHANNEL_ID_TO_USERNAME_STORAGE_KEY),
            reserved_usernames: Map::new(RESERVED_USERNAMES_STORAGE_KEY),
            channel_metadata: Map::new(CHANNEL_METADATA_STORAGE_KEY),
        }
    }

    pub fn add_reserved_usernames(
        &self,
        store: &mut dyn Storage,
        usernames: Vec<(UserName, Addr)>,
    ) -> Result<(), ChannelError> {
        for username in usernames {
            self.reserved_usernames
                .save(store, username.0.clone(), &username.1)
                .map_err(|_| ChannelError::SaveReservedUsernamesFailed {})?;
        }
        Ok(())
    }
    pub fn remove_reserved_usernames(
        &self,
        store: &mut dyn Storage,
        usernames: Vec<UserName>,
    ) -> Result<(), ChannelError> {
        for username in usernames {
            // return error if username does not exist
            if !self.reserved_usernames.has(store, username.clone()) {
                return Err(ChannelError::UsernameNotReserved {});
            }
            self.reserved_usernames.remove(store, username.clone());
        }
        Ok(())
    }

    // Query methods
    pub fn get_channel_details(
        &self,
        store: &dyn Storage,
        channel_id: ChannelId,
    ) -> Result<ChannelDetails, ChannelError> {
        self.channel_details
            .load(store, channel_id)
            .map_err(|_| ChannelError::ChannelIdNotFound {})
    }

    pub fn update_collaborators(
        &self,
        store: &mut dyn Storage,
        channel_id: ChannelId,
        collaborators: Vec<Addr>,
    ) -> Result<(), ChannelError> {
        let mut channel_details = self
            .channel_details
            .load(store, channel_id.clone())
            .map_err(|_| ChannelError::ChannelIdNotFound {})?;

        channel_details.collaborators = collaborators;
        self.channel_details
            .save(store, channel_id, &channel_details)
            .map_err(|_| ChannelError::SaveChannelDetailsFailed {})?;

        Ok(())
    }

    pub fn get_channels_list(
        &self,
        store: &dyn Storage,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<Vec<ChannelDetails>> {
        let limit = limit.unwrap_or(25) as usize;
        let start = start_after.map(Bound::exclusive);

        self.channel_details
            .range(store, start, None, Order::Ascending)
            .take(limit)
            .map(|item| item.map(|(_, details)| details))
            .collect()
    }

    pub fn get_channel_id(
        &self,
        store: &dyn Storage,
        user_name: UserName,
    ) -> Result<ChannelId, ChannelError> {
        self.username_to_channel_id
            .load(store, user_name)
            .map_err(|_| ChannelError::UserNameNotFound {})
    }

    // Mutation methods
    pub fn add_channel(
        &self,
        store: &mut dyn Storage,
        channel_id: ChannelId,
        user_name: UserName,
        channel_details: ChannelDetails,
        channel_metadata: ChannelMetadata,
    ) -> Result<(), ChannelError> {
        // Check if the channel ID or username already exists
        if self.channel_details.has(store, channel_id.clone()) {
            return Err(ChannelError::ChannelIdAlreadyExists {});
        }
        if self.username_to_channel_id.has(store, user_name.clone()) {
            return Err(ChannelError::UserNameAlreadyTaken {});
        }

        // Save the details and mappings
        self.channel_details
            .save(store, channel_id.clone(), &channel_details)
            .map_err(|_| ChannelError::SaveChannelDetailsFailed {})?;
        self.username_to_channel_id
            .save(store, user_name.clone(), &channel_id)
            .map_err(|_| ChannelError::SaveChannelDetailsFailed {})?;
        self.channel_id_to_username
            .save(store, channel_id.clone(), &user_name)
            .map_err(|_| ChannelError::SaveChannelDetailsFailed {})?;
        self.channel_metadata
            .save(store, channel_id, &channel_metadata)
            .map_err(|_| ChannelError::SaveChannelDetailsFailed {})?;

        Ok(())
    }
    pub fn get_channel_metadata(
        &self,
        store: &dyn Storage,
        channel_id: ChannelId,
    ) -> Result<ChannelMetadata, ChannelError> {
        self.channel_metadata
            .load(store, channel_id)
            .map_err(|_| ChannelError::ChannelIdNotFound {})
    }
    pub fn update_channel_metadata(
        &self,
        store: &mut dyn Storage,
        channel_id: ChannelId,
        channel_metadata: ChannelMetadata,
    ) -> Result<(), ChannelError> {
        if !self.channel_metadata.has(store, channel_id.clone()) {
            return Err(ChannelError::ChannelIdNotFound {});
        }

        self.channel_metadata
            .save(store, channel_id, &channel_metadata)
            .map_err(|_| ChannelError::SaveChannelDetailsFailed {})?;

        Ok(())
    }

    pub fn delete_channel(
        &self,
        store: &mut dyn Storage,
        channel_id: ChannelId,
    ) -> Result<(), ChannelError> {
        if !self.channel_details.has(store, channel_id.clone()) {
            return Err(ChannelError::ChannelIdNotFound {});
        }

        let user_name = self
            .channel_id_to_username
            .load(store, channel_id.clone())
            .map_err(|_| ChannelError::UserNameNotFound {})?;

        // Remove channel details and mappings
        self.channel_details.remove(store, channel_id.clone());
        self.username_to_channel_id.remove(store, user_name.clone());
        self.channel_id_to_username.remove(store, channel_id);

        Ok(())
    }

    pub fn get_channel_id_from_username(
        &self,
        store: &mut dyn Storage,
        user_name: UserName,
    ) -> Result<ChannelId, ChannelError> {
        self.username_to_channel_id
            .load(store, user_name)
            .map_err(|_| ChannelError::UserNameNotFound {})
    }

    pub fn get_username_from_channel_id(
        &self,
        store: &mut dyn Storage,
        channel_id: ChannelId,
    ) -> Result<UserName, ChannelError> {
        self.channel_id_to_username
            .load(store, channel_id)
            .map_err(|_| ChannelError::ChannelIdNotFound {})
    }
    pub fn get_reserved_usernames(
        &self,
        store: &dyn Storage,
        start_after: Option<UserName>,
        limit: Option<u32>,
    ) -> StdResult<Vec<(UserName, Addr)>> {
        let limit = limit.unwrap_or(25).min(25) as usize;
        let start = start_after.map(Bound::exclusive);

        self.reserved_usernames
            .range(store, start, None, Order::Ascending)
            .take(limit)
            .collect()
    }
    pub fn get_reserved_status(
        &self,
        store: &dyn Storage,
        username: UserName,
    ) -> StdResult<Option<Addr>> {
        let reserved_address = self.reserved_usernames.load(store, username);
        if reserved_address.is_err() {
            return Ok(None);
        }
        Ok(Some(reserved_address.unwrap()))
    }
    pub fn update_channel_details(
        &self,
        store: &mut dyn Storage,
        channel_id: ChannelId,
        channel_details: ChannelDetails,
    ) -> Result<(), ChannelError> {
        if !self.channel_details.has(store, channel_id.clone()) {
            return Err(ChannelError::ChannelIdNotFound {});
        }

        self.channel_details
            .save(store, channel_id, &channel_details)
            .map_err(|_| ChannelError::SaveChannelDetailsFailed {})?;

        Ok(())
    }
}
