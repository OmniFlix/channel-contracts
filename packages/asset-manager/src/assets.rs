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
        channel_id: ChannelId,
        publish_id: PublishId,
    ) -> Result<Asset, AssetError> {
        self.assets
            .load(store, (channel_id, publish_id))
            .map_err(|_| AssetError::AssetNotFound {})
    }

    pub fn remove_asset(
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
        publish_id: PublishId,
    ) -> Result<bool, AssetError> {
        let exists = self
            .assets
            .range(store, None, None, Order::Descending) // range in reverse order
            .map(|item| item.map(|((_, pub_id), _)| pub_id)) // Only deal with publish_id
            .filter(|result| result.is_ok() && result.as_ref().unwrap() == &publish_id) // filter by publish_id
            .next(); // Stop as soon as we find the first match

        Ok(exists.is_some()) // If we found any, return true
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
}
