use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal};
pub type ChannelId = String;
pub type UserName = String;

#[cw_serde]
pub struct ChannelDetails {
    pub channel_id: String,
    pub user_name: String,
    pub onft_id: String,
    pub payment_address: Addr,
}

#[cw_serde]
pub struct ChannelMetadata {
    pub channel_name: String,
    pub description: Option<String>,
    pub profile_picture: Option<String>,
    pub banner_picture: Option<String>,
}

#[cw_serde]
// This is a struct that used for each onft's additional data field.
pub struct ChannelOnftData {
    pub onft_id: String,
    pub channel_id: String,
    pub user_name: String,
}
#[cw_serde]
pub struct ChannelCollaborator {
    pub role: Role,
    pub share: Decimal,
}

#[cw_serde]
pub enum Role {
    Admin,
    Publisher,
    Moderator,
}

impl Role {
    // Alternative implementation using numeric values
    pub fn privilege_level(&self) -> u8 {
        match self {
            Role::Admin => 3,
            Role::Moderator => 2,
            Role::Publisher => 1,
        }
    }

    pub fn has_sufficient_privileges(&self, required_role: &Role) -> bool {
        self.privilege_level() >= required_role.privilege_level()
    }
}
