use cosmwasm_std::{Addr, Binary};
use omniflix_channel_types::msg::{ExecuteMsg, InstantiateMsg, ReservedUsername};

pub fn get_channel_instantiate_msg(admin: Addr) -> InstantiateMsg {
    InstantiateMsg {
        channel_creation_fee: vec![],
        fee_collector: admin.clone(),
        protocol_admin: admin.clone(),
        accepted_tip_denoms: vec!["uflix".to_string()],
        channels_collection_id: "Channels".to_string(),
        channels_collection_name: "Channels".to_string(),
        channels_collection_symbol: "CH".to_string(),
        reserved_usernames: vec![ReservedUsername {
            username: "reserved".to_string(),
            address: None,
        }],
    }
}

pub struct CreateChannelMsgBuilder {
    salt: Binary,
    user_name: String,
    description: String,
    channel_name: String,
    banner_picture: Option<String>,
    profile_picture: Option<String>,
    payment_address: Addr,
}

impl CreateChannelMsgBuilder {
    pub fn new(user_name: &str, payment_address: Addr) -> Self {
        Self {
            salt: Binary::from("salt".as_bytes()),
            user_name: user_name.to_string(),
            description: "Default description".to_string(),
            channel_name: user_name.to_string(), // Default to the same as user_name
            banner_picture: None,
            profile_picture: None,
            payment_address,
        }
    }

    pub fn description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn channel_name(mut self, channel_name: String) -> Self {
        self.channel_name = channel_name;
        self
    }

    pub fn banner_picture(mut self, banner_picture: String) -> Self {
        self.banner_picture = Some(banner_picture);
        self
    }

    pub fn profile_picture(mut self, profile_picture: String) -> Self {
        self.profile_picture = Some(profile_picture);
        self
    }

    pub fn salt(mut self, salt: Binary) -> Self {
        self.salt = salt;
        self
    }

    pub fn build(self) -> ExecuteMsg {
        ExecuteMsg::ChannelCreate {
            salt: self.salt,
            user_name: self.user_name,
            description: Some(self.description),
            banner_picture: self.banner_picture,
            profile_picture: self.profile_picture,
            channel_name: self.channel_name,
            payment_address: self.payment_address,
        }
    }
}
