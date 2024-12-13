use channel_types::msg::{ExecuteMsg, InstantiateMsg};
use cosmwasm_std::{Addr, Binary};

pub fn get_channel_instantiate_msg(admin: Addr) -> InstantiateMsg {
    InstantiateMsg {
        channel_creation_fee: vec![],
        fee_collector: admin.clone(),
        admin: admin,
        channels_collection_id: "Channels".to_string(),
        channels_collection_name: "Channels".to_string(),
        channels_collection_symbol: "CH".to_string(),
        reserved_usernames: vec!["reserved".to_string()],
    }
}

pub fn get_channel_create_msg(user_name: String) -> ExecuteMsg {
    ExecuteMsg::ChannelCreate {
        salt: Binary::from("salt".as_bytes()),
        user_name: user_name,
        description: "Creator 1 Description".to_string(),
        collaborators: None,
    }
}
