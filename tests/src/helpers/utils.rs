use cosmwasm_std::{from_json, Addr, Coin, CosmosMsg, MemoryStorage, Storage};
use cw_multi_test::AppResponse;
use omniflix_std::types::omniflix::onft::v1beta1::{Collection, MsgCreateDenom, MsgMintOnft};
use testing::app::OmniflixApp;

pub fn get_contract_address_from_res(res: AppResponse) -> String {
    res.events
        .iter()
        .find(|e| e.ty == "instantiate")
        .unwrap()
        .attributes
        .iter()
        .find(|a| a.key == "_contract_address")
        .unwrap()
        .value
        .clone()
}

pub fn query_onft_collection(storage: &MemoryStorage, minter_address: String) -> Collection {
    let key = format!("collections:{}:{}", "collection", minter_address);
    let collection = storage.get(key.as_bytes()).unwrap();
    let collection_details: Collection = from_json(collection).unwrap();
    collection_details
}
pub fn mint_to_address(app: &mut OmniflixApp, to_address: String, amount: Vec<Coin>) {
    app.init_modules(|router, _, storage| {
        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked(to_address.clone()),
                amount.clone(),
            )
            .unwrap()
    });
}
pub fn get_event_attribute(res: AppResponse, event_type: &str, attribute_key: &str) -> String {
    res.events
        .iter()
        .find(|e| e.ty == event_type) // Find the event by type
        .and_then(|e| {
            e.attributes.iter().find(|attr| attr.key == attribute_key) // Find the attribute by key
        })
        .map_or(String::new(), |attr| attr.value.clone()) // Return value or empty string
}

/// Generates a CosmosMsg to create a denom
pub fn create_denom_msg(
    sender: String,
    denom_id: String,
    name: Option<String>, // Optional: if not provided, default is used
) -> CosmosMsg {
    let create_denom_msg = MsgCreateDenom {
        data: "data".to_string(),
        sender: sender.clone(),
        creation_fee: None,
        description: "description".to_string(),
        id: denom_id.clone(),
        name: name.unwrap_or_else(|| "Media asset collection".to_string()),
        preview_uri: "preview_uri".to_string(),
        royalty_receivers: vec![],
        schema: "schema".to_string(),
        symbol: "symbol".to_string(),
        uri: "uri".to_string(),
        uri_hash: "uri_hash".to_string(),
    };

    create_denom_msg.into() // Convert to CosmosMsg
}

/// Generates a CosmosMsg to mint an ONFT
pub fn mint_onft_msg(
    denom_id: String,
    onft_id: String, // Renamed from asset_id to onft_id
    recipient: String,
) -> CosmosMsg {
    let mint_onft_msg = MsgMintOnft {
        denom_id: denom_id.clone(),
        id: onft_id.clone(),
        nsfw: false, // Default is false
        recipient: recipient.clone(),
        ..Default::default() // Use default values for remaining fields
    };

    mint_onft_msg.into() // Convert to CosmosMsg
}
