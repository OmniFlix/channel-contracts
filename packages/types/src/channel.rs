use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal};
pub type ChannelId = String;
pub type UserName = String;

#[cw_serde]
pub struct ChannelDetails {
    pub channel_id: String,
    pub user_name: String,
    pub onft_id: String,
    pub collaborators: Vec<Addr>,
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
    pub expires_at: Option<u64>,
    pub share: Decimal,
}

#[cw_serde]
pub enum Role {
    Publisher,
    Moderator,
}
