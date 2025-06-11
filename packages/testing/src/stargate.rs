use anyhow::{bail, Result};
use cosmwasm_std::{from_json, to_json_binary, Addr, Api, Binary, BlockInfo, Querier, Storage};
use cw_multi_test::{error::AnyResult, AppResponse, CosmosRouter, Stargate};
use omniflix_std::types::omniflix::onft::v1beta1::{
    Collection, Denom, MsgCreateDenom, MsgMintOnft, QueryOnftRequest, QueryOnftResponse,
};
use omniflix_std::types::{
    cosmos::base::v1beta1::Coin,
    omniflix::onft::v1beta1::{Onft, Params, QueryParamsResponse},
};
use prost::{DecodeError, Message};

const COLLECTION_PREFIX: &str = "collection";

pub struct StargateKeeper {}

impl StargateKeeper {}

impl Stargate for StargateKeeper {
    fn execute_stargate<ExecC, QueryC>(
        &self,
        _api: &dyn Api,
        storage: &mut dyn Storage,
        _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        _block: &BlockInfo,
        sender: Addr,
        type_url: String,
        value: Binary,
    ) -> AnyResult<AppResponse> {
        if type_url == *"/OmniFlix.onft.v1beta1.MsgCreateDenom" {
            let parsed_msg: Result<MsgCreateDenom, DecodeError> = Message::decode(value.as_slice());
            if let Ok(msg) = parsed_msg {
                let collection = Collection {
                    denom: Some(Denom {
                        creator: sender.to_string(),
                        data: msg.data,
                        name: msg.name,
                        id: msg.id.clone(),
                        preview_uri: msg.preview_uri,
                        description: msg.description,
                        schema: msg.schema,
                        symbol: msg.symbol,
                        uri: msg.uri,
                        uri_hash: msg.uri_hash,
                        royalty_receivers: msg.royalty_receivers,
                    }),
                    onfts: vec![],
                };
                let key = format!("collections:{}:{}", COLLECTION_PREFIX, msg.id);
                let serialized_collection =
                    to_json_binary(&collection).expect("Failed to serialize Collection");
                storage.set(key.as_bytes(), &serialized_collection);
            }
        }
        if type_url == *"/OmniFlix.onft.v1beta1.MsgMintONFT" {
            let parsed_msg: Result<MsgMintOnft, DecodeError> = Message::decode(value.as_slice());
            if let Ok(msg) = parsed_msg {
                let key = format!("collections:{}:{}", COLLECTION_PREFIX, msg.denom_id);
                let serialized_collection = storage.get(key.as_bytes());
                let mut collection: Collection = from_json(serialized_collection.unwrap())
                    .expect("Failed to deserialize Collection");
                let onft = Onft {
                    id: msg.id,
                    created_at: None,
                    nsfw: msg.nsfw,
                    owner: msg.recipient,
                    data: msg.data,
                    transferable: msg.transferable,
                    extensible: msg.extensible,
                    metadata: msg.metadata,
                    royalty_share: msg.royalty_share,
                };
                println!("onft: {:?}", onft);
                collection.onfts.push(onft);
                let serialized_collection =
                    to_json_binary(&collection).expect("Failed to serialize Collection");
                storage.set(key.as_bytes(), &serialized_collection);
            };
        }
        Ok(AppResponse::default())
    }

    fn query_stargate(
        &self,
        _api: &dyn Api,
        storage: &dyn Storage,
        _querier: &dyn Querier,
        _block: &BlockInfo,
        path: String,
        data: Binary,
    ) -> AnyResult<Binary> {
        if path == *"/OmniFlix.onft.v1beta1.Query/Params" {
            let params = QueryParamsResponse {
                params: Some(Params {
                    denom_creation_fee: Some(Coin {
                        denom: "uflix".to_string(),
                        amount: "1000000".to_string(),
                    }),
                }),
            };
            return Ok(to_json_binary(&params)?);
        }
        if path == *"/OmniFlix.onft.v1beta1.Query/ONFT" {
            let query_msg: Result<QueryOnftRequest, DecodeError> = Message::decode(data.as_slice());
            if let Ok(msg) = query_msg {
                let key = format!("collections:{}:{}", COLLECTION_PREFIX, msg.denom_id);
                let serialized_collection = storage.get(key.as_bytes());
                if serialized_collection.is_none() {
                    return Ok(to_json_binary(&QueryOnftResponse { onft: None })?);
                }
                let collection: Collection = from_json(serialized_collection.unwrap())
                    .expect("Failed to deserialize Collection");
                let onft = collection
                    .onfts
                    .iter()
                    .find(|onft| onft.id == msg.id)
                    .expect("Onft not found");
                let response = QueryOnftResponse {
                    onft: Some(onft.clone()),
                };
                return Ok(to_json_binary(&response)?);
            }
        }
        Ok(data)
    }

    fn execute_any<ExecC, QueryC>(
        &self,
        _api: &dyn Api,
        _storage: &mut dyn Storage,
        _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        _block: &BlockInfo,
        _sender: Addr,
        _msg: cosmwasm_std::AnyMsg,
    ) -> AnyResult<AppResponse>
    where
        ExecC: cosmwasm_std::CustomMsg + serde::de::DeserializeOwned + 'static,
        QueryC: cosmwasm_std::CustomQuery + serde::de::DeserializeOwned + 'static,
    {
        bail!("execute_any not implemented")
    }

    fn query_grpc(
        &self,
        _api: &dyn Api,
        _storage: &dyn Storage,
        _querier: &dyn Querier,
        _block: &BlockInfo,
        _request: cosmwasm_std::GrpcQuery,
    ) -> AnyResult<Binary> {
        bail!("query_grpc not implemented")
    }
}
