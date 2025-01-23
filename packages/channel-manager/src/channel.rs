use cosmwasm_std::{Addr, Decimal, Order, StdResult, Storage};
use cw_storage_plus::{Bound, Map};

use crate::error::ChannelError;
use omniflix_channel_types::channel::{
    ChannelCollaborator, ChannelDetails, ChannelId, ChannelMetadata, UserName,
};

const CHANNEL_DETAILS_STORAGE_KEY: &str = "channel_details";
const CHANNEL_METADATA_STORAGE_KEY: &str = "channel_metadata";
const USERNAME_TO_CHANNEL_ID_STORAGE_KEY: &str = "username_to_channel_id";
const CHANNEL_ID_TO_USERNAME_STORAGE_KEY: &str = "channel_id_to_username";
const RESERVED_USERNAMES_STORAGE_KEY: &str = "reserved_usernames";
const CHANNEL_COLLABORATORS_MAP_STORAGE_KEY: &str = "channel_collaborators_map";

pub struct ChannelsManager {
    pub channel_details: Map<ChannelId, ChannelDetails>,
    pub channel_metadata: Map<ChannelId, ChannelMetadata>,
    pub username_to_channel_id: Map<UserName, ChannelId>,
    pub channel_id_to_username: Map<ChannelId, UserName>,
    pub reserved_usernames: Map<UserName, Addr>,
    pub channel_collaborators: Map<(ChannelId, Addr), ChannelCollaborator>,
}

impl ChannelsManager {
    pub const fn new() -> Self {
        ChannelsManager {
            channel_details: Map::new(CHANNEL_DETAILS_STORAGE_KEY),
            username_to_channel_id: Map::new(USERNAME_TO_CHANNEL_ID_STORAGE_KEY),
            channel_id_to_username: Map::new(CHANNEL_ID_TO_USERNAME_STORAGE_KEY),
            reserved_usernames: Map::new(RESERVED_USERNAMES_STORAGE_KEY),
            channel_metadata: Map::new(CHANNEL_METADATA_STORAGE_KEY),
            channel_collaborators: Map::new(CHANNEL_COLLABORATORS_MAP_STORAGE_KEY),
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

    pub fn add_collaborator(
        &self,
        store: &mut dyn Storage,
        channel_id: ChannelId,
        address: Addr,
        collaborator: ChannelCollaborator,
    ) -> Result<(), ChannelError> {
        // Load the channel collaborator, if exist return error
        if self
            .channel_collaborators
            .has(store, (channel_id.clone(), address.clone()))
        {
            return Err(ChannelError::CollaboratorExists {});
        }
        // Get all the channel collaborators
        let channel_collaborators: Vec<(Addr, ChannelCollaborator)> = self
            .channel_collaborators
            .prefix(channel_id.clone())
            .range(store, None, None, Order::Ascending)
            .collect::<Result<Vec<_>, _>>()
            .unwrap_or_default();
        // If the total share is bigger than 1, return error
        let total_share = channel_collaborators
            .iter()
            .map(|(_, collaborator)| collaborator.share)
            .sum::<Decimal>();

        if total_share + collaborator.share > Decimal::one() {
            return Err(ChannelError::InvalidSharePercentage {});
        }
        // Save the collaborator
        self.channel_collaborators
            .save(store, (channel_id.clone(), address), &collaborator)
            .map_err(|_| ChannelError::SaveChannelDetailsFailed {})?;
        Ok(())
    }

    pub fn remove_collaborator(
        &self,
        store: &mut dyn Storage,
        channel_id: ChannelId,
        address: Addr,
    ) -> Result<(), ChannelError> {
        // Check if the collaborator exists
        if !self
            .channel_collaborators
            .has(store, (channel_id.clone(), address.clone()))
        {
            return Err(ChannelError::CollaboratorNotFound {});
        }
        // Remove the collaborator
        self.channel_collaborators
            .remove(store, (channel_id, address));
        Ok(())
    }
    pub fn get_collaborator_shares(
        &self,
        store: &dyn Storage,
        channel_id: ChannelId,
    ) -> Result<Vec<(Addr, Decimal)>, ChannelError> {
        let channel_collaborators: Vec<(Addr, ChannelCollaborator)> = self
            .channel_collaborators
            .prefix(channel_id)
            .range(store, None, None, Order::Ascending)
            .collect::<Result<Vec<_>, _>>()
            .unwrap_or_default();
        let shares = channel_collaborators
            .iter()
            .map(|(addr, collaborator)| (addr.clone(), collaborator.share))
            .collect();
        Ok(shares)
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

#[cfg(test)]
mod tests {
    use omniflix_channel_types::channel::Role;

    use super::*;
    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::Uint128;

    #[test]
    fn test_collaborator_operations() {
        let mut deps = mock_dependencies();
        let channels = ChannelsManager::new();
        let channel_id1 = "channel1".to_string();
        let channel_id2 = "channel2".to_string();
        let addr1 = Addr::unchecked("addr1");
        let addr2 = Addr::unchecked("addr2");
        let addr3 = Addr::unchecked("addr3");
        let addr4 = Addr::unchecked("addr4");

        // Test adding collaborators to channel 1
        let collab1 = ChannelCollaborator {
            expires_at: None,
            role: Role::Moderator,
            share: Decimal::from_ratio(
                Uint128::from(500000000000000000u128),
                Uint128::from(1000000000000000000u128),
            ), // 50%
        };
        let result = channels.add_collaborator(
            &mut deps.storage,
            channel_id1.clone(),
            addr1.clone(),
            collab1,
        );
        assert!(result.is_ok());

        let collab2 = ChannelCollaborator {
            expires_at: None,
            role: Role::Publisher,
            share: Decimal::from_ratio(
                Uint128::from(500000000000000000u128),
                Uint128::from(1000000000000000000u128),
            ), // 50%
        };
        let result = channels.add_collaborator(
            &mut deps.storage,
            channel_id1.clone(),
            addr2.clone(),
            collab2,
        );
        assert!(result.is_ok());

        // Test adding collaborators to channel 2
        let collab3 = ChannelCollaborator {
            expires_at: None,
            role: Role::Moderator,
            share: Decimal::from_ratio(
                Uint128::from(300000000000000000u128),
                Uint128::from(1000000000000000000u128),
            ), // 30%
        };
        let result = channels.add_collaborator(
            &mut deps.storage,
            channel_id2.clone(),
            addr3.clone(),
            collab3,
        );
        assert!(result.is_ok());

        let collab4 = ChannelCollaborator {
            expires_at: None,
            role: Role::Publisher,
            share: Decimal::from_ratio(
                Uint128::from(700000000000000000u128),
                Uint128::from(1000000000000000000u128),
            ), // 70%
        };
        let result = channels.add_collaborator(
            &mut deps.storage,
            channel_id2.clone(),
            addr4.clone(),
            collab4,
        );
        assert!(result.is_ok());

        // Verify shares for channel 1
        let shares1 = channels
            .get_collaborator_shares(&deps.storage, channel_id1.clone())
            .unwrap();
        assert_eq!(shares1.len(), 2);
        assert!(shares1.contains(&(
            addr1.clone(),
            Decimal::from_ratio(
                Uint128::from(500000000000000000u128),
                Uint128::from(1000000000000000000u128)
            )
        )));
        assert!(shares1.contains(&(
            addr2.clone(),
            Decimal::from_ratio(
                Uint128::from(500000000000000000u128),
                Uint128::from(1000000000000000000u128)
            )
        )));

        // Verify shares for channel 2
        let shares2 = channels
            .get_collaborator_shares(&deps.storage, channel_id2.clone())
            .unwrap();
        assert_eq!(shares2.len(), 2);
        assert!(shares2.contains(&(
            addr3.clone(),
            Decimal::from_ratio(
                Uint128::from(300000000000000000u128),
                Uint128::from(1000000000000000000u128)
            )
        )));
        assert!(shares2.contains(&(
            addr4.clone(),
            Decimal::from_ratio(
                Uint128::from(700000000000000000u128),
                Uint128::from(1000000000000000000u128)
            )
        )));

        // Test removing collaborator from channel 1 doesn't affect channel 2
        let result =
            channels.remove_collaborator(&mut deps.storage, channel_id1.clone(), addr1.clone());
        assert!(result.is_ok());

        // Verify channel 1 shares updated
        let shares1_after = channels
            .get_collaborator_shares(&deps.storage, channel_id1.clone())
            .unwrap();
        assert_eq!(shares1_after.len(), 1);
        assert!(shares1_after.contains(&(
            addr2.clone(),
            Decimal::from_ratio(
                Uint128::from(500000000000000000u128),
                Uint128::from(1000000000000000000u128)
            )
        )));

        // Verify channel 2 shares unchanged
        let shares2_after = channels
            .get_collaborator_shares(&deps.storage, channel_id2.clone())
            .unwrap();
        assert_eq!(shares2_after.len(), 2);
        assert!(shares2_after.contains(&(
            addr3.clone(),
            Decimal::from_ratio(
                Uint128::from(300000000000000000u128),
                Uint128::from(1000000000000000000u128)
            )
        )));
        assert!(shares2_after.contains(&(
            addr4.clone(),
            Decimal::from_ratio(
                Uint128::from(700000000000000000u128),
                Uint128::from(1000000000000000000u128)
            )
        )));
    }

    #[test]
    fn test_add_collaborator_overflow() {
        let mut deps = mock_dependencies();
        let channels = ChannelsManager::new();
        let channel_id = "channel1".to_string();
        let addr1 = Addr::unchecked("addr1");
        let addr2 = Addr::unchecked("addr2");
        let addr3 = Addr::unchecked("addr3");

        // Add first collaborator
        let collab1 = ChannelCollaborator {
            expires_at: None,
            role: Role::Moderator,
            share: Decimal::from_ratio(
                Uint128::from(500000000000000000u128),
                Uint128::from(1000000000000000000u128),
            ),
        };
        let result = channels.add_collaborator(
            &mut deps.storage,
            channel_id.clone(),
            addr1.clone(),
            collab1,
        );
        assert!(result.is_ok());

        // Now we are at 50%
        // Add another collaborator with 50% share. This should work
        let collab2 = ChannelCollaborator {
            expires_at: None,
            role: Role::Moderator,
            share: Decimal::from_ratio(
                Uint128::from(500000000000000000u128),
                Uint128::from(1000000000000000000u128),
            ),
        };
        let result = channels.add_collaborator(
            &mut deps.storage,
            channel_id.clone(),
            addr2.clone(),
            collab2,
        );
        assert!(result.is_ok());

        // Add another collaborator with 50% share. This should overflow
        let collab3 = ChannelCollaborator {
            expires_at: None,
            role: Role::Moderator,
            share: Decimal::from_ratio(
                Uint128::from(500000000000000000u128),
                Uint128::from(1000000000000000000u128),
            ),
        };
        let result = channels.add_collaborator(
            &mut deps.storage,
            channel_id.clone(),
            addr3.clone(),
            collab3,
        );
        assert_eq!(result.unwrap_err(), ChannelError::InvalidSharePercentage {});
    }

    #[test]
    fn test_duplicate_collaborator() {
        let mut deps = mock_dependencies();
        let channels = ChannelsManager::new();
        let channel_id = "channel1".to_string();
        let addr1 = Addr::unchecked("addr1");

        // Add first collaborator
        let collab = ChannelCollaborator {
            expires_at: None,
            role: Role::Moderator,
            share: Decimal::from_ratio(
                Uint128::from(500000000000000000u128),
                Uint128::from(1000000000000000000u128),
            ),
        };
        let result = channels.add_collaborator(
            &mut deps.storage,
            channel_id.clone(),
            addr1.clone(),
            collab.clone(),
        );
        assert!(result.is_ok());

        // Try to add same collaborator again
        let result = channels.add_collaborator(
            &mut deps.storage,
            channel_id.clone(),
            addr1.clone(),
            collab.clone(),
        );
        assert!(matches!(result, Err(ChannelError::CollaboratorExists {})));
    }
}
