use cosmwasm_schema::cw_serde;

use crate::channel::ChannelId;

pub type PublishId = String;
/// Define `AssetKey` as a tuple of `ChannelId` and `PublishId`.
pub type AssetKey = (ChannelId, PublishId);

#[cw_serde]
pub struct Playlist {
    pub playlist_name: String,
    pub assets: Vec<AssetKey>,
}

#[cw_serde]
pub enum AssetType {
    Nft {
        collection_id: String,
        onft_id: String,
    },
    OffChain {
        media_uri: String,
        name: String,
        description: String,
    },
}

// Implement to string for AssetType
impl std::fmt::Display for AssetType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AssetType::Nft {
                collection_id,
                onft_id,
            } => write!(f, "NFT: {} {}", collection_id, onft_id),
            AssetType::OffChain {
                media_uri,
                name,
                description,
            } => write!(f, "OffChain: {} {} {}", media_uri, name, description),
        }
    }
}

#[cw_serde]
pub struct Asset {
    pub channel_id: String,
    pub publish_id: String,
    pub asset_type: AssetType,
    pub is_visible: bool,
}
