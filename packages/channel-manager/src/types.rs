use cosmwasm_schema::cw_serde;

use crate::error::ChannelError;

pub type ChannelId = String;
pub type UserName = String;

#[cw_serde]
pub struct ChannelDetails {
    pub channel_id: String,
    pub user_name: String,
    pub description: String,
    pub onft_id: String,
}

impl ChannelDetails {
    pub fn new(
        channel_id: String,
        user_name: String,
        description: String,
        onft_id: String,
    ) -> Self {
        Self {
            channel_id,
            user_name,
            description,
            onft_id,
        }
    }

    pub fn validate(&self) -> Result<(), ChannelError> {
        if self.user_name.len() < 3 || self.user_name.len() > 32 {
            return Err(ChannelError::InvalidUserName {});
        }
        if self.description.len() < 3 || self.description.len() > 256 {
            return Err(ChannelError::InvalidDescription {});
        }
        Ok(())
    }
}

#[cw_serde]
pub struct ChannelOnftData {
    pub onft_id: String,
    pub channel_id: String,
    pub user_name: String,
}
