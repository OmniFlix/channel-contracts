use cosmwasm_std::{Addr, Binary};
use omniflix_channel_types::msg::{ExecuteMsg, InstantiateMsg, ReservedUsername};

pub fn get_channel_instantiate_msg(admin: Addr) -> InstantiateMsg {
    InstantiateMsg {
        channel_creation_fee: vec![],
        fee_collector: admin.clone(),
        admin: admin,
        channels_collection_id: "Channels".to_string(),
        channels_collection_name: "Channels".to_string(),
        channels_collection_symbol: "CH".to_string(),
        reserved_usernames: vec![ReservedUsername {
            username: "reserved".to_string(),
            address: None,
        }],
    }
}

pub fn get_channel_create_msg(user_name: String) -> ExecuteMsg {
    ExecuteMsg::ChannelCreate {
        salt: Binary::from("salt".as_bytes()),
        user_name: user_name,
        description: "Creator 1 Description".to_string(),
        collaborators: None,
        banner_picture: None,
        profile_picture: None,
        channel_name: "Creator1".to_string(),
    }
}
