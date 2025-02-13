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
pub enum AssetSource {
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

// Implement to string for AssetSource
impl std::fmt::Display for AssetSource {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AssetSource::Nft {
                collection_id,
                onft_id,
            } => write!(f, "NFT: {} {}", collection_id, onft_id),
            AssetSource::OffChain {
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
    pub asset_source: AssetSource,
    pub is_visible: bool,
}
