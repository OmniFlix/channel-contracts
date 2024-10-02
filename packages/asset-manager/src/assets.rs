use crate::error::AssetError;
use cosmwasm_std::{Order, StdResult, Storage};
use cw_storage_plus::{Bound, Map};

use crate::types::Asset;

type ChannelId = String;
type PublishId = String;

pub struct Assets<'a> {
    pub assets: Map<'a, (ChannelId, PublishId), Asset>,
}

impl Assets<'_> {
    pub const fn new() -> Self {
        Assets {
            assets: Map::new("assets"),
        }
    }

    pub fn add_asset(
        &self,
        store: &mut dyn Storage,
        channel_id: ChannelId,
        asset: Asset,
    ) -> Result<(), AssetError> {
        if self
            .assets
            .has(store, (channel_id.clone(), asset.publish_id.clone()))
        {
            return Err(AssetError::AssetAlreadyExists {});
        }

        self.assets
            .save(store, (channel_id, asset.publish_id.clone()), &asset)
            .map_err(|_| AssetError::SaveAssetError {})?;

        Ok(())
    }

    pub fn get_asset(
        &self,
        store: &dyn Storage,
        asset_channel_id: ChannelId,
        publish_id: PublishId,
    ) -> Result<Asset, AssetError> {
        self.assets
            .load(store, (asset_channel_id, publish_id))
            .map_err(|_| AssetError::AssetNotFound {})
    }

    pub fn delete_asset(
        &self,
        store: &mut dyn Storage,
        channel_id: ChannelId,
        publish_id: PublishId,
    ) -> Result<(), AssetError> {
        if self
            .assets
            .load(store, (channel_id.clone(), publish_id.clone()))
            .is_err()
        {
            return Err(AssetError::AssetNotFound {});
        }

        self.assets.remove(store, (channel_id, publish_id));
        Ok(())
    }

    pub fn get_all_assets(
        &self,
        store: &dyn Storage,
        channel_id: ChannelId,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<Vec<Asset>> {
        let limit = limit.unwrap_or(25) as usize;
        let start = start_after.map(Bound::exclusive);

        let assets = self
            .assets
            .prefix(channel_id)
            .range(store, start, None, Order::Ascending)
            .take(limit)
            .map(|item| item.map(|(_, asset)| asset))
            .collect::<StdResult<Vec<Asset>>>()?;

        Ok(assets)
    }
    pub fn asset_exists(
        &self,
        store: &dyn Storage,
        channel_id: ChannelId,
        publish_id: PublishId,
    ) -> Result<bool, AssetError> {
        let exists = self.assets.load(store, (channel_id, publish_id)).is_ok();
        Ok(exists)
    }
    pub fn update_asset(
        &self,
        store: &mut dyn Storage,
        channel_id: ChannelId,
        publish_id: PublishId,
        updated_asset: Asset,
    ) -> Result<(), AssetError> {
        if self
            .assets
            .load(store, (channel_id.clone(), publish_id.clone()))
            .is_err()
        {
            return Err(AssetError::AssetNotFound {});
        }

        self.assets
            .save(store, (channel_id, publish_id), &updated_asset)
            .map_err(|_| AssetError::SaveAssetError {})?;

        Ok(())
    }
    pub fn delete_assets_by_channel_id(
        &self,
        store: &mut dyn Storage,
        channel_id: ChannelId,
    ) -> Result<(), AssetError> {
        self.assets.prefix(channel_id.clone()).clear(store, None);
        Ok(())
    }
}

// Test delete_assets_by_channel_id
#[cfg(test)]

mod tests {
    use super::*;
    use crate::types::Asset;
    use cosmwasm_std::testing::MockStorage;

    #[test]
    fn test_delete_assets_by_channel_id() {
        let mut storage = MockStorage::new();
        let assets = Assets::new();
        // Generate 20 assets and add them to the storage
        for i in 0..20 {
            let asset = Asset {
                publish_id: format!("asset{}", i),
                channel_id: "channel1".to_string(),
                collection_id: "collection1".to_string(),
                is_visible: true,
                onft_id: "onft1".to_string(),
            };
            assets
                .add_asset(&mut storage, "channel1".to_string(), asset)
                .unwrap();
        }

        // Check if all assets are added
        let all_assets = assets
            .get_all_assets(&storage, "channel1".to_string(), None, None)
            .unwrap();
        assert_eq!(all_assets.len(), 20);

        // Delete all assets
        assets
            .delete_assets_by_channel_id(&mut storage, "channel1".to_string())
            .unwrap();
    }
}
