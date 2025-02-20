use cosmwasm_schema::cw_serde;

use crate::channel::ChannelId;

pub type PublishId = String;
/// Used to identify an asset over a channel and all contract
pub type AssetKey = (ChannelId, PublishId);
/// Used to identify a flags of assets
pub type FlagKey = (String, AssetKey);

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

#[cw_serde]
pub enum Flag {
    NSFW,
    Explicit,
    Spam,
    Hateful,
    Other(String),
}
impl std::fmt::Display for Flag {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Flag::NSFW => write!(f, "NSFW"),
            Flag::Explicit => write!(f, "Explicit"),
            Flag::Spam => write!(f, "Spam"),
            Flag::Hateful => write!(f, "Hateful"),
            Flag::Other(s) => write!(f, "Other: {}", s),
        }
    }
}

impl Flag {
    pub fn to_key(&self) -> String {
        match self {
            Flag::NSFW => "n".to_string(),
            Flag::Explicit => "e".to_string(),
            Flag::Spam => "s".to_string(),
            Flag::Hateful => "h".to_string(),
            Flag::Other(_) => "o".to_string(),
        }
    }
    pub fn values() -> Vec<Flag> {
        vec![
            Flag::NSFW,
            Flag::Explicit,
            Flag::Spam,
            Flag::Hateful,
            Flag::Other(String::new()), // Assuming you want to include this variant
        ]
    }
}
