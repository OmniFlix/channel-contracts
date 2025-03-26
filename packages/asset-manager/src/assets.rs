use crate::error::AssetError;
use cosmwasm_std::{Order, StdResult, Storage};
use cw_storage_plus::{Bound, Map};

use omniflix_channel_types::{
    asset::{Asset, AssetKey, AssetMetadata, Flag, FlagKey, PublishId},
    channel::ChannelId,
    msg::{AssetResponse, FlagInfo},
};

pub struct AssetsManager {
    pub assets: Map<AssetKey, Asset>,
    pub asset_metadata: Map<AssetKey, AssetMetadata>,
    pub flags: Map<FlagKey, u64>,
}

const PAGINATION_LIMIT: u32 = 50;

impl Default for AssetsManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetsManager {
    /// Create a new `Assets` instance.
    pub const fn new() -> Self {
        AssetsManager {
            assets: Map::new("assets"),
            asset_metadata: Map::new("asset_metadata"),
            flags: Map::new("flags"),
        }
    }

    /// Add a new asset, ensuring it does not already exist.
    pub fn add_asset(
        &self,
        store: &mut dyn Storage,
        key: AssetKey,
        asset: Asset,
        metadata: AssetMetadata,
    ) -> Result<(), AssetError> {
        if self.assets.has(store, key.clone()) {
            return Err(AssetError::AssetAlreadyExists {});
        }

        self.assets
            .save(store, key.clone(), &asset)
            .map_err(|_| AssetError::SaveAssetError {})?;
        self.asset_metadata
            .save(store, key, &metadata)
            .map_err(|_| AssetError::SaveAssetMetadataError {})?;

        Ok(())
    }

    /// Retrieve an asset by its key.
    pub fn get_asset(&self, store: &dyn Storage, key: AssetKey) -> Result<Asset, AssetError> {
        self.assets
            .load(store, key)
            .map_err(|_| AssetError::AssetNotFound {})
    }

    pub fn get_asset_metadata(
        &self,
        store: &dyn Storage,
        key: AssetKey,
    ) -> Result<AssetMetadata, AssetError> {
        self.asset_metadata
            .load(store, key)
            .map_err(|_| AssetError::AssetMetadataNotFound {})
    }

    /// Delete an asset by its key.
    pub fn delete_assets(
        &self,
        store: &mut dyn Storage,
        keys: Vec<AssetKey>,
    ) -> Result<(), AssetError> {
        for key in keys {
            if self.assets.load(store, key.clone()).is_err() {
                return Err(AssetError::AssetNotFound {});
            }
            self.assets.remove(store, key.clone());
            self.asset_metadata.remove(store, key);
        }
        Ok(())
    }

    pub fn get_all_assets(
        &self,
        store: &dyn Storage,
        channel_id: ChannelId,
        start_after: Option<PublishId>,
        limit: Option<u32>,
    ) -> StdResult<Vec<AssetResponse>> {
        let limit = limit.unwrap_or(PAGINATION_LIMIT).min(PAGINATION_LIMIT) as usize;
        let start = start_after.map(Bound::exclusive);

        let assets = self
            .assets
            .prefix(channel_id.clone())
            .range(store, start, None, Order::Ascending)
            .take(limit)
            .map(|result| {
                result.map(|(_, asset)| {
                    let flags = self
                        .get_all_flags_for_asset(
                            store,
                            channel_id.clone(),
                            asset.publish_id.clone(),
                        )
                        .unwrap_or_default();
                    let metadata = self
                        .get_asset_metadata(store, (channel_id.clone(), asset.publish_id.clone()))
                        .unwrap_or_default();

                    AssetResponse {
                        asset: asset.clone(),
                        flags,
                        metadata,
                    }
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .unwrap_or_default();

        Ok(assets)
    }

    /// Check if an asset exists by its key.
    pub fn asset_exists(&self, store: &dyn Storage, key: AssetKey) -> bool {
        self.assets.has(store, key)
    }

    /// Update an existing asset by its key.
    pub fn update_asset(
        &self,
        store: &mut dyn Storage,
        key: AssetKey,
        updated_asset: Asset,
    ) -> Result<(), AssetError> {
        if !self.assets.has(store, key.clone()) {
            return Err(AssetError::AssetNotFound {});
        }

        self.assets
            .save(store, key, &updated_asset)
            .map_err(|_| AssetError::SaveAssetError {})?;

        Ok(())
    }

    pub fn update_asset_metadata(
        &self,
        store: &mut dyn Storage,
        key: AssetKey,
        metadata: AssetMetadata,
    ) -> Result<(), AssetError> {
        if !self.assets.has(store, key.clone()) {
            return Err(AssetError::AssetNotFound {});
        }

        self.asset_metadata
            .save(store, key, &metadata)
            .map_err(|_| AssetError::SaveAssetMetadataError {})?;
        Ok(())
    }

    /// Delete all assets for a specific channel.
    pub fn delete_assets_by_channel_id(
        &self,
        store: &mut dyn Storage,
        channel_id: ChannelId,
    ) -> Result<(), AssetError> {
        self.assets.prefix(channel_id.clone()).clear(store, None);
        self.asset_metadata
            .prefix(channel_id.clone())
            .clear(store, None);
        Ok(())
    }

    pub fn add_flag(
        &self,
        store: &mut dyn Storage,
        channel_id: ChannelId,
        publish_id: PublishId,
        flag: Flag,
    ) -> Result<(), AssetError> {
        let key = (flag.to_key(), (channel_id, publish_id));
        let flagged_count = self.flags.load(store, key.clone()).unwrap_or(0);
        // save the new count
        self.flags
            .save(store, key, &(flagged_count + 1))
            .map_err(|_| AssetError::SaveFlagError {})?;
        Ok(())
    }

    pub fn get_flag_count(
        &self,
        store: &dyn Storage,
        channel_id: ChannelId,
        publish_id: PublishId,
        flag: Flag,
    ) -> Result<u64, AssetError> {
        // Check if the asset exists
        if !self
            .assets
            .has(store, (channel_id.clone(), publish_id.clone()))
        {
            return Err(AssetError::AssetNotFound {});
        }
        let key = (flag.to_key(), (channel_id.clone(), publish_id.clone()));
        let flag_count = self.flags.load(store, key).unwrap_or(0);
        Ok(flag_count)
    }

    pub fn get_all_flags_for_asset(
        &self,
        store: &dyn Storage,
        channel_id: ChannelId,
        publish_id: PublishId,
    ) -> Result<Vec<FlagInfo>, AssetError> {
        let mut flags = Vec::new();
        for flag in Flag::values() {
            let flag_count =
                self.get_flag_count(store, channel_id.clone(), publish_id.clone(), flag.clone())?;
            flags.push(FlagInfo {
                flag: flag.clone(),
                count: flag_count,
            });
        }
        Ok(flags)
    }
    pub fn remove_all_flags(&self, store: &mut dyn Storage) -> Result<(), AssetError> {
        self.flags.clear(store);
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use cosmwasm_std::testing::MockStorage;
//     use omniflix_channel_types::asset::AssetSource;

//     #[test]
//     fn test_get_all_assets_with_limit() {
//         let mut storage = MockStorage::new();
//         let assets = AssetsManager::new();

//         let channel_id = "channel1".to_string();

//         // Generate 100 assets and add them to the storage
//         for i in 0..100 {
//             let publish_id = format!("asset{}", i);
//             let asset = Asset {
//                 publish_id: publish_id.clone(),
//                 channel_id: channel_id.clone(),
//                 is_visible: true,
//                 asset_source: AssetSource::Nft {
//                     collection_id: "collection_id".to_string(),
//                     onft_id: "onft_id".to_string(),
//                 },
//                 name: "name".to_string(),
//                 description: "description".to_string(),
//                 media_uri: "http://www.media.com".to_string(),
//             };
//             assets
//                 .add_asset(&mut storage, (channel_id.clone(), publish_id), asset)
//                 .unwrap();
//         }

//         // Test with a limit of 25
//         let assets_page_1 = assets
//             .get_all_assets(&storage, channel_id.clone(), None, Some(25))
//             .unwrap();
//         assert_eq!(assets_page_1.len(), 25); // Should return exactly 25 assets

//         // Test with a limit of 15
//         let assets_page_2 = assets
//             .get_all_assets(&storage, channel_id.clone(), None, Some(15))
//             .unwrap();
//         assert_eq!(assets_page_2.len(), 15); // Should return exactly 15 assets

//         // Test with a limit > MAX_LIMIT
//         let assets_page_3 = assets
//             .get_all_assets(
//                 &storage,
//                 channel_id.clone(),
//                 None,
//                 Some(PAGINATION_LIMIT + 1),
//             )
//             .unwrap();
//         assert_eq!(assets_page_3.len(), PAGINATION_LIMIT as usize); // Should return exactly 25 assets
//     }

//     #[test]
//     fn test_get_all_assets_with_start_after() {
//         let mut storage = MockStorage::new();
//         let assets = AssetsManager::new();

//         let channel_id = "channel1".to_string();

//         // Generate 100 assets and add them to the storage
//         for i in 0..100 {
//             let publish_id = format!("asset{}", i);
//             let asset = Asset {
//                 publish_id: publish_id.clone(),
//                 channel_id: channel_id.clone(),
//                 is_visible: true,
//                 asset_source: AssetSource::Nft {
//                     collection_id: "collection_id".to_string(),
//                     onft_id: "onft_id".to_string(),
//                 },
//                 name: "name".to_string(),
//                 description: "description".to_string(),
//                 media_uri: "http://www.media.com".to_string(),
//             };
//             assets
//                 .add_asset(&mut storage, (channel_id.clone(), publish_id), asset)
//                 .unwrap();
//         }

//         // Test start_after with publish_id "asset50"
//         let assets_start_after = assets
//             .get_all_assets(
//                 &storage,
//                 channel_id.clone(),
//                 Some("asset50".to_string()),
//                 Some(20),
//             )
//             .unwrap();
//         assert_eq!(assets_start_after.len(), 20); // Should return assets starting after "asset50"

//         // Ensure the first asset in the result is "asset51"
//         assert_eq!(
//             assets_start_after[0].asset.publish_id,
//             "asset51".to_string()
//         );
//     }

//     #[test]
//     fn test_get_all_assets_with_limit_and_start_after() {
//         let mut storage = MockStorage::new();
//         let assets = AssetsManager::new();

//         let channel_id = "channel1".to_string();

//         // Generate 100 assets and add them to the storage
//         for i in 0..100 {
//             let publish_id = format!("asset{}", i);
//             let asset = Asset {
//                 publish_id: publish_id.clone(),
//                 channel_id: channel_id.clone(),
//                 is_visible: true,
//                 asset_source: AssetSource::Nft {
//                     collection_id: "collection_id".to_string(),
//                     onft_id: "onft_id".to_string(),
//                 },
//                 name: "name".to_string(),
//                 description: "description".to_string(),
//                 media_uri: "http://www.media.com".to_string(),
//             };
//             assets
//                 .add_asset(&mut storage, (channel_id.clone(), publish_id), asset)
//                 .unwrap();
//         }

//         // Test with a limit of 30 and start_after with publish_id "asset40"
//         let assets_with_start_after_and_limit = assets
//             .get_all_assets(
//                 &storage,
//                 channel_id.clone(),
//                 Some("asset40".to_string()),
//                 Some(20),
//             )
//             .unwrap();
//         assert_eq!(assets_with_start_after_and_limit.len(), 20); // Should return 20 assets

//         // Ensure the first asset in the result is "asset41"
//         assert_eq!(
//             assets_with_start_after_and_limit[0].asset.publish_id,
//             "asset41".to_string()
//         );
//     }
//     #[test]
//     fn test_add_asset() {
//         let mut storage = MockStorage::new();
//         let assets = AssetsManager::new();

//         let channel_id = "channel1".to_string();
//         let publish_id = "asset1".to_string();
//         let asset = Asset {
//             publish_id: publish_id.clone(),
//             channel_id: channel_id.clone(),
//             is_visible: true,
//             asset_source: AssetSource::Nft {
//                 collection_id: "collection_id".to_string(),
//                 onft_id: "onft_id".to_string(),
//             },
//             name: "name".to_string(),
//             description: "description".to_string(),
//             media_uri: "http://www.media.com".to_string(),
//         };

//         // Add the asset
//         let add_result = assets.add_asset(
//             &mut storage,
//             (channel_id.clone(), publish_id.clone()),
//             asset.clone(),
//         );
//         assert!(add_result.is_ok());

//         // Try adding the same asset again (should fail)
//         let add_result_again =
//             assets.add_asset(&mut storage, (channel_id.clone(), publish_id), asset);
//         assert!(add_result_again.is_err()); // AssetAlreadyExists error
//     }

//     #[test]
//     fn test_get_asset() {
//         let mut storage = MockStorage::new();
//         let assets = AssetsManager::new();

//         let channel_id = "channel1".to_string();
//         let publish_id = "asset1".to_string();
//         let asset = Asset {
//             publish_id: publish_id.clone(),
//             channel_id: channel_id.clone(),
//             is_visible: true,
//             asset_source: AssetSource::Nft {
//                 collection_id: "collection_id".to_string(),
//                 onft_id: "onft_id".to_string(),
//             },
//             name: "name".to_string(),
//             description: "description".to_string(),
//             media_uri: "http://www.media.com".to_string(),
//         };

//         // Add the asset to storage
//         assets
//             .add_asset(
//                 &mut storage,
//                 (channel_id.clone(), publish_id.clone()),
//                 asset,
//             )
//             .unwrap();

//         // Retrieve the asset
//         let retrieved_asset = assets
//             .get_asset(&storage, (channel_id.clone(), publish_id.clone()))
//             .unwrap();
//         assert_eq!(retrieved_asset.publish_id, publish_id);
//     }

//     #[test]
//     fn test_delete_asset() {
//         let mut storage = MockStorage::new();
//         let assets = AssetsManager::new();

//         let channel_id = "channel1".to_string();
//         let publish_id = "asset1".to_string();
//         let asset = Asset {
//             publish_id: publish_id.clone(),
//             channel_id: channel_id.clone(),
//             is_visible: true,
//             asset_source: AssetSource::Nft {
//                 collection_id: "collection_id".to_string(),
//                 onft_id: "onft_id".to_string(),
//             },
//             name: "name".to_string(),
//             description: "description".to_string(),
//             media_uri: "http://www.media.com".to_string(),
//         };

//         // Add the asset
//         assets
//             .add_asset(
//                 &mut storage,
//                 (channel_id.clone(), publish_id.clone()),
//                 asset,
//             )
//             .unwrap();

//         // Delete the asset
//         let delete_result =
//             assets.delete_assets(&mut storage, vec![(channel_id.clone(), publish_id.clone())]);
//         assert!(delete_result.is_ok());

//         // Try to get the deleted asset (should fail)
//         let get_result = assets.get_asset(&storage, (channel_id.clone(), publish_id.clone()));
//         assert!(get_result.is_err()); // AssetNotFound error
//     }

//     #[test]
//     fn test_update_asset() {
//         let mut storage = MockStorage::new();
//         let assets = AssetsManager::new();

//         let channel_id = "channel1".to_string();
//         let publish_id = "asset1".to_string();
//         let asset = Asset {
//             publish_id: publish_id.clone(),
//             channel_id: channel_id.clone(),
//             is_visible: true,
//             asset_source: AssetSource::Nft {
//                 collection_id: "collection_id".to_string(),
//                 onft_id: "onft_id".to_string(),
//             },
//             name: "name".to_string(),
//             description: "description".to_string(),
//             media_uri: "http://www.media.com".to_string(),
//         };

//         // Add the asset
//         assets
//             .add_asset(
//                 &mut storage,
//                 (channel_id.clone(), publish_id.clone()),
//                 asset,
//             )
//             .unwrap();

//         // Update the asset
//         let updated_asset = Asset {
//             publish_id: publish_id.clone(),
//             channel_id: channel_id.clone(),
//             is_visible: false, // Changing visibility
//             asset_source: AssetSource::Nft {
//                 collection_id: "new_collection".to_string(),
//                 onft_id: "new_onft_id".to_string(),
//             },
//             name: "new_name".to_string(),
//             description: "new_description".to_string(),
//             media_uri: "http://www.media1.com".to_string(),
//         };
//         let update_result = assets.update_asset(
//             &mut storage,
//             (channel_id.clone(), publish_id.clone()),
//             updated_asset,
//         );
//         assert!(update_result.is_ok());

//         // Retrieve the updated asset
//         let retrieved_asset = assets
//             .get_asset(&storage, (channel_id.clone(), publish_id.clone()))
//             .unwrap();
//         assert!(!retrieved_asset.is_visible); // Asset should be updated
//     }

//     #[test]
//     fn test_delete_assets_by_channel_id() {
//         let mut storage = MockStorage::new();
//         let assets = AssetsManager::new();

//         let channel_id = "channel1".to_string();
//         let publish_id1 = "asset1".to_string();
//         let publish_id2 = "asset2".to_string();
//         let asset1 = Asset {
//             publish_id: publish_id1.clone(),
//             channel_id: channel_id.clone(),
//             is_visible: true,
//             asset_source: AssetSource::Nft {
//                 collection_id: "collection_id".to_string(),
//                 onft_id: "onft_id".to_string(),
//             },
//             name: "name".to_string(),
//             description: "description".to_string(),
//             media_uri: "http://www.media.com".to_string(),
//         };
//         let asset2 = Asset {
//             publish_id: publish_id2.clone(),
//             channel_id: channel_id.clone(),
//             is_visible: true,
//             asset_source: AssetSource::Nft {
//                 collection_id: "collection_id".to_string(),
//                 onft_id: "onft_id".to_string(),
//             },
//             name: "name".to_string(),
//             description: "description".to_string(),
//             media_uri: "http://www.media.com".to_string(),
//         };

//         // Add the assets
//         assets
//             .add_asset(
//                 &mut storage,
//                 (channel_id.clone(), publish_id1.clone()),
//                 asset1,
//             )
//             .unwrap();
//         assets
//             .add_asset(
//                 &mut storage,
//                 (channel_id.clone(), publish_id2.clone()),
//                 asset2,
//             )
//             .unwrap();

//         // Delete all assets for the channel
//         let delete_result = assets.delete_assets_by_channel_id(&mut storage, channel_id.clone());
//         assert!(delete_result.is_ok());

//         // Try to retrieve the assets (should fail)
//         let get_result1 = assets.get_asset(&storage, (channel_id.clone(), publish_id1.clone()));
//         let get_result2 = assets.get_asset(&storage, (channel_id.clone(), publish_id2.clone()));
//         assert!(get_result1.is_err());
//         assert!(get_result2.is_err());
//     }

//     #[test]
//     fn test_asset_exists() {
//         let mut storage = MockStorage::new();
//         let assets = AssetsManager::new();

//         let channel_id = "channel1".to_string();
//         let publish_id = "asset1".to_string();
//         let asset = Asset {
//             publish_id: publish_id.clone(),
//             channel_id: channel_id.clone(),
//             is_visible: true,
//             asset_source: AssetSource::Nft {
//                 collection_id: "collection_id".to_string(),
//                 onft_id: "onft_id".to_string(),
//             },
//             name: "name".to_string(),
//             description: "description".to_string(),
//             media_uri: "http://www.media.com".to_string(),
//         };

//         // Add the asset
//         assets
//             .add_asset(
//                 &mut storage,
//                 (channel_id.clone(), publish_id.clone()),
//                 asset,
//             )
//             .unwrap();

//         // Check if the asset exists
//         let exists = assets.asset_exists(&storage, (channel_id.clone(), publish_id.clone()));
//         assert!(exists);

//         // Check for an asset that doesn't exist
//         let exists_not_found =
//             assets.asset_exists(&storage, (channel_id.clone(), "nonexistent".to_string()));
//         assert!(!exists_not_found);
//     }
// }
