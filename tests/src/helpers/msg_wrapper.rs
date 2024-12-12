// // Instantiate Channel Contract
// let instantiate_msg = InstantiateMsg {
//     admin: setup_response.test_accounts.admin.clone(),
//     channel_creation_fee: vec![],
//     fee_collector: setup_response.test_accounts.admin,
//     channels_collection_id: "Channels".to_string(),
//     channels_collection_name: "Channels".to_string(),
//     channels_collection_symbol: "CH".to_string(),
// };

use channel_types::msg::InstantiateMsg;
use cosmwasm_std::Addr;

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
