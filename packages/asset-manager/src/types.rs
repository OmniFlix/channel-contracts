use cosmwasm_schema::cw_serde;

use crate::error::AssetError;

#[cw_serde]
pub struct Playlist {
    pub playlist_name: String,
    pub assets: Vec<Asset>,
}

#[cw_serde]
pub enum AssetType {
    NFT {
        collection_id: String,
        onft_id: String,
    },
    OffChain {
        ipfs_link: String,
        name: String,
        description: String,
    },
}

// Implement to string for AssetType
impl std::fmt::Display for AssetType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AssetType::NFT {
                collection_id,
                onft_id,
            } => write!(f, "NFT: {} {}", collection_id, onft_id),
            AssetType::OffChain {
                ipfs_link,
                name,
                description,
            } => write!(f, "OffChain: {} {} {}", ipfs_link, name, description),
        }
    }
}

// Implement validate for AssetType
impl AssetType {
    pub fn validate(&self) -> Result<(), AssetError> {
        match self {
            AssetType::NFT {
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
                ipfs_link,
                name,
                description,
            } => {
                if ipfs_link.is_empty() {
                    return Err(AssetError::IpfsLinkCannotBeEmpty {});
                }
                if ipfs_link.len() > 256 {
                    return Err(AssetError::IpfsLinkTooLong {});
                }
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
