use cosmwasm_schema::cw_serde;

use crate::error::AssetError;

#[cw_serde]
pub struct Playlist {
    pub playlist_name: String,
    pub assets: Vec<Asset>,
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

// Implement validate for AssetType
impl AssetType {
    pub fn validate(&self) -> Result<(), AssetError> {
        match self {
            AssetType::Nft {
                collection_id,
                onft_id,
            } => {
                if collection_id.is_empty() {
                    return Err(AssetError::CollectionIdCannotBeEmpty {});
                }
                if onft_id.is_empty() {
                    return Err(AssetError::OnftIdCannotBeEmpty {});
                }
            }
            AssetType::OffChain {
                media_uri,
                name,
                description,
            } => {
                if media_uri.is_empty() {
                    return Err(AssetError::MediaUriCannotBeEmpty {});
                }

                if media_uri.len() > 255 {
                    return Err(AssetError::MediaUriTooLong {});
                };
                if name.is_empty() {}
                if description.is_empty() {
                    return Err(AssetError::DescriptionCannotBeEmpty {});
                }
            }
        }
        Ok(())
    }
}

#[cw_serde]
pub struct Asset {
    pub channel_id: String,
    pub publish_id: String,
    pub asset_type: AssetType,
    pub is_visible: bool,
}
