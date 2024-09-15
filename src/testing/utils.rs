use crate::testing::app::OmniflixApp;
use cosmwasm_std::{from_json, Coin, MemoryStorage, Storage};
use cw_multi_test::{AppResponse, BankSudo, SudoMsg};
use omniflix_std::types::omniflix::onft::v1beta1::Collection;

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
    app.sudo(SudoMsg::Bank(BankSudo::Mint { to_address, amount }))
        .unwrap();
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
