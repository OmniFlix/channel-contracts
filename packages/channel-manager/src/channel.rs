use cosmwasm_std::{Addr, Decimal, Order, StdResult, Storage};
use cw_storage_plus::{Bound, Map};

use crate::error::ChannelError;
use omniflix_channel_types::channel::{
    ChannelCollaborator, ChannelDetails, ChannelId, ChannelMetadata, UserName,
};

const CHANNEL_DETAILS: &str = "cd"; // channel_details
const CHANNEL_METADATA: &str = "cm"; // channel_metadata
const USERNAME_TO_CHANNEL_ID: &str = "u2i"; // username_to_channel_id
const CHANNEL_ID_TO_USERNAME: &str = "i2u"; // channel_id_to_username
const RESERVED_USERNAMES: &str = "ru"; // reserved_usernames
const CHANNEL_COLLABORATORS: &str = "col"; // channel_collaborators
const TOTAL_COLLABORATOR_SHARES: &str = "tcs"; // total_collaborator_shares
const TOTAL_UNIQUE_COLLABORATOR_LIMIT: u32 = 10;

pub struct ChannelsManager {
    pub channel_details: Map<ChannelId, ChannelDetails>,
    pub channel_metadata: Map<ChannelId, ChannelMetadata>,
    pub username_to_channel_id: Map<UserName, ChannelId>,
    pub channel_id_to_username: Map<ChannelId, UserName>,
    pub reserved_usernames: Map<UserName, Addr>,
    pub channel_collaborators: Map<(ChannelId, Addr), ChannelCollaborator>,
    pub total_collaborator_shares: Map<ChannelId, Decimal>,
}

impl ChannelsManager {
    pub const fn new() -> Self {
        ChannelsManager {
            channel_details: Map::new(CHANNEL_DETAILS),
            username_to_channel_id: Map::new(USERNAME_TO_CHANNEL_ID),
            channel_id_to_username: Map::new(CHANNEL_ID_TO_USERNAME),
            reserved_usernames: Map::new(RESERVED_USERNAMES),
            channel_metadata: Map::new(CHANNEL_METADATA),
            channel_collaborators: Map::new(CHANNEL_COLLABORATORS),
            total_collaborator_shares: Map::new(TOTAL_COLLABORATOR_SHARES),
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

    pub fn get_channel_details(
        &self,
        store: &dyn Storage,
        channel_id: ChannelId,
    ) -> Result<ChannelDetails, ChannelError> {
        self.channel_details
            .load(store, channel_id)
            .map_err(|_| ChannelError::ChannelIdNotFound {})
    }

    pub fn update_payment_address(
        &self,
        store: &mut dyn Storage,
        channel_id: ChannelId,
        payment_address: Addr,
    ) -> Result<(), ChannelError> {
        let mut channel_details = self.get_channel_details(store, channel_id.clone())?;
        channel_details.payment_address = payment_address;
        self.channel_details
            .save(store, channel_id.clone(), &channel_details)
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

    pub fn get_channel_id(
        &self,
        store: &dyn Storage,
        user_name: UserName,
    ) -> Result<ChannelId, ChannelError> {
        self.username_to_channel_id
            .load(store, user_name)
            .map_err(|_| ChannelError::UserNameNotFound {})
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

    pub fn add_collaborator(
        &self,
        store: &mut dyn Storage,
        channel_id: ChannelId,
        address: Addr,
        collaborator: ChannelCollaborator,
    ) -> Result<(), ChannelError> {
        // Check if collaborator already exists
        if self
            .channel_collaborators
            .has(store, (channel_id.clone(), address.clone()))
        {
            return Err(ChannelError::CollaboratorExists {});
        }

        // Calculate new total share
        let current_total = self
            .total_collaborator_shares
            .load(store, channel_id.clone())
            .unwrap_or(Decimal::zero());
        let new_total = current_total + collaborator.share;

        // Check if the number of unique collaborators exceeds the limit
        let unique_collaborators = self
            .channel_collaborators
            .prefix(channel_id.clone())
            .keys(store, None, None, Order::Ascending)
            .count();
        if unique_collaborators >= TOTAL_UNIQUE_COLLABORATOR_LIMIT as usize {
            return Err(ChannelError::TotalUniqueCollaboratorsLimitExceeded {});
        }

        // Validate total share doesn't exceed 100%
        if new_total > Decimal::one() {
            return Err(ChannelError::InvalidSharePercentage {});
        }

        // Save collaborator and update total shares
        self.channel_collaborators
            .save(store, (channel_id.clone(), address), &collaborator)
            .map_err(|_| ChannelError::SaveChannelDetailsFailed {})?;

        self.total_collaborator_shares
            .save(store, channel_id, &new_total)
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
        let collaborator = self
            .channel_collaborators
            .load(store, (channel_id.clone(), address.clone()))
            .unwrap();
        // Remove the collaborator
        self.channel_collaborators
            .remove(store, (channel_id.clone(), address.clone()));
        // Update total shares
        let current_total = self
            .total_collaborator_shares
            .load(store, channel_id.clone())
            .unwrap_or(Decimal::zero());
        let new_total = current_total - collaborator.share;
        self.total_collaborator_shares
            .save(store, channel_id, &new_total)
            .map_err(|_| ChannelError::SaveChannelDetailsFailed {})?;
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

    pub fn is_collaborator(
        &self,
        store: &dyn Storage,
        channel_id: ChannelId,
        sender: Addr,
    ) -> Result<bool, ChannelError> {
        let collaborator = self
            .channel_collaborators
            .has(store, (channel_id.clone(), sender.clone()));
        Ok(collaborator)
    }
    pub fn get_collaborator(
        &self,
        store: &dyn Storage,
        channel_id: ChannelId,
        sender: Addr,
    ) -> Result<ChannelCollaborator, ChannelError> {
        let collaborator = self
            .channel_collaborators
            .load(store, (channel_id.clone(), sender.clone()));
        if collaborator.is_err() {
            return Err(ChannelError::CollaboratorNotFound {});
        }
        Ok(collaborator.unwrap())
    }

    pub fn get_channel_collaborators(
        &self,
        store: &dyn Storage,
        channel_id: ChannelId,
    ) -> Result<Vec<(Addr, ChannelCollaborator)>, ChannelError> {
        let channel_collaborators: Vec<(Addr, ChannelCollaborator)> = self
            .channel_collaborators
            .prefix(channel_id)
            .range(store, None, None, Order::Ascending)
            .collect::<Result<Vec<_>, _>>()
            .unwrap_or_default();
        Ok(channel_collaborators)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::mock_dependencies;
    use omniflix_channel_types::channel::Role;

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
            role: Role::Moderator,
            share: Decimal::percent(50), // 50%
        };
        let result = channels.add_collaborator(
            &mut deps.storage,
            channel_id1.clone(),
            addr1.clone(),
            collab1,
        );
        assert!(result.is_ok());

        let collab2 = ChannelCollaborator {
            role: Role::Publisher,
            share: Decimal::percent(50), // 50%
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
            role: Role::Moderator,
            share: Decimal::percent(30), // 30%
        };
        let result = channels.add_collaborator(
            &mut deps.storage,
            channel_id2.clone(),
            addr3.clone(),
            collab3,
        );
        assert!(result.is_ok());

        let collab4 = ChannelCollaborator {
            role: Role::Publisher,
            share: Decimal::percent(70), // 70%
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
        assert!(shares1.contains(&(addr1.clone(), Decimal::percent(50))));
        assert!(shares1.contains(&(addr2.clone(), Decimal::percent(50))));

        // Verify shares for channel 2
        let shares2 = channels
            .get_collaborator_shares(&deps.storage, channel_id2.clone())
            .unwrap();
        assert_eq!(shares2.len(), 2);
        assert!(shares2.contains(&(addr3.clone(), Decimal::percent(30))));
        assert!(shares2.contains(&(addr4.clone(), Decimal::percent(70))));

        // Test removing collaborator from channel 1 doesn't affect channel 2
        let result =
            channels.remove_collaborator(&mut deps.storage, channel_id1.clone(), addr1.clone());
        assert!(result.is_ok());

        // Verify channel 1 shares updated
        let shares1_after = channels
            .get_collaborator_shares(&deps.storage, channel_id1.clone())
            .unwrap();
        assert_eq!(shares1_after.len(), 1);
        assert!(shares1_after.contains(&(addr2.clone(), Decimal::percent(50))));

        // Verify channel 2 shares unchanged
        let shares2_after = channels
            .get_collaborator_shares(&deps.storage, channel_id2.clone())
            .unwrap();
        assert_eq!(shares2_after.len(), 2);
        assert!(shares2_after.contains(&(addr3.clone(), Decimal::percent(30))));
        assert!(shares2_after.contains(&(addr4.clone(), Decimal::percent(70))));
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
            role: Role::Moderator,
            share: Decimal::percent(50),
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
            role: Role::Moderator,
            share: Decimal::percent(50),
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
            role: Role::Moderator,
            share: Decimal::percent(50),
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
            role: Role::Moderator,
            share: Decimal::percent(50),
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
