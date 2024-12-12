use asset_manager::types::{Asset, AssetType, Playlist};
use channel_manager::types::ChannelDetails;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, Coin};

use crate::config::ChannelConractConfig;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Addr,
    pub fee_collector: Addr,
    pub channels_collection_id: String,
    pub channels_collection_name: String,
    pub channels_collection_symbol: String,
    pub channel_creation_fee: Vec<Coin>,
    pub reserved_usernames: Vec<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Pauses all channel-related operations. Only callable by a pauser.
    Pause {},

    /// Resumes all paused operations. Only callable by a pauser.
    Unpause {},

    /// Updates the list of accounts allowed to pause and unpause the contract.
    /// Only callable by an admin.
    SetPausers {
        /// A list of addresses to be set as pausers.
        pausers: Vec<String>,
    },

    /// Publishes an asset to a channel. The contract will generate and store a publish ID.
    /// Only callable by the channel owner or a collaborator.
    Publish {
        asset_type: AssetType,
        /// A salt value used for unique identification.
        salt: Binary,
        /// The ID of the channel where the asset is published.
        channel_id: String,
        /// (Optional) The name of the playlist where the asset is added.
        playlist_name: Option<String>,
        /// A flag indicating if the asset is visible to the public.
        is_visible: bool,
    },

    /// Unpublishes an asset from a channel. The publish ID and related asset details will
    /// be removed from the contract state. Only callable by the channel owner or a collaborator.
    Unpublish {
        /// The ID of the publish to be removed.
        publish_id: String,
        /// The ID of the channel where the asset is unpublished.
        channel_id: String,
    },

    /// Updates the details of a published asset, including its visibility status.
    /// Only callable by the channel owner.
    AssetUpdateDetails {
        /// The ID of the publish to be updated.
        publish_id: String,
        /// The ID of the channel where the asset is published.
        channel_id: String,
        /// The new visibility status of the asset.
        is_visible: bool,
    },

    /// Creates a new playlist in the specified channel. The playlist name must be unique
    /// within the channel. Only callable by the channel owner or a collaborator.
    PlaylistCreate {
        /// The unique name of the playlist.
        playlist_name: String,
        /// The ID of the channel where the playlist is created.
        channel_id: String,
    },

    /// Deletes an existing playlist from the channel.
    /// Only callable by the channel owner or a collaborator.
    PlaylistDelete {
        /// The name of the playlist to be deleted.
        playlist_name: String,
        /// The ID of the channel where the playlist exists.
        channel_id: String,
    },

    /// Adds an asset to a playlist. The asset must already be published in the channel
    /// and must be visible. Only callable by the channel owner or a collaborator.
    PlaylistAddAsset {
        /// The publish ID of the asset to be added.
        publish_id: String,
        /// The ID of the channel where the asset is currently published.
        asset_channel_id: String,
        /// The ID of the channel where the playlist exists.
        channel_id: String,
        /// The name of the playlist where the asset will be added.
        playlist_name: String,
    },

    /// Removes an asset from a playlist.
    /// Only callable by the channel owner or a collaborator.
    PlaylistRemoveAsset {
        /// The publish ID of the asset to be removed.
        publish_id: String,
        /// The ID of the channel where the playlist exists.
        channel_id: String,
        /// The name of the playlist where the asset is being removed.
        playlist_name: String,
    },

    /// Refreshes a playlist by removing all assets that are either unpublished
    /// or no longer visible. Only callable by the channel owner or a collaborator.
    PlaylistRefresh {
        /// The ID of the channel where the playlist exists.
        channel_id: String,
        /// The name of the playlist to be refreshed.
        playlist_name: String,
    },

    /// Creates a new channel. The contract will generate a channel ID and mint an NFT
    /// for the owner. The owner can add collaborators to the channel.
    /// The owner must pay the `channel_creation_fee` to create a channel.
    ChannelCreate {
        /// A salt value used for unique identification.
        salt: Binary,
        /// The user name of the channel owner.
        user_name: String,
        /// A description of the channel.
        description: String,
        /// (Optional) A list of collaborator addresses for the channel.
        collaborators: Option<Vec<String>>,
    },

    /// Deletes an existing channel. The channel ID and related details will be removed
    /// from the contract state. Only callable by the channel owner.
    ChannelDelete {
        /// The ID of the channel to be deleted.
        channel_id: String,
    },

    /// Updates the details of an existing channel. Only callable by the channel owner.
    ChannelUpdateDetails {
        /// The ID of the channel to be updated.
        channel_id: String,
        /// The new description for the channel.
        description: String,
    },

    /// Updates the configuration of the contract, including the channel creation fee,
    /// admin, and fee collector. Only callable by the protocol admin.
    SetConfig {
        /// (Optional) The new channel creation fee.
        channel_creation_fee: Option<Vec<Coin>>,
        /// (Optional) The new admin address.
        admin: Option<String>,
        /// (Optional) The new fee collector address.
        fee_collector: Option<String>,
    },

    /// Adds a reserved usernames to the contract. Reserved usernames cannot be used
    /// as channel names. Only callable by the protocol admin.
    AddReservedUsernames {
        /// A list of usernames to be added to the reserved list.
        usernames: Vec<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(bool)]
    IsPaused {},
    #[returns(Vec<String>)]
    Pausers {},
    #[returns(ChannelDetails)]
    ChannelDetails {
        channel_id: Option<String>,
        user_name: Option<String>,
    },
    #[returns(Vec<ChannelDetails>)]
    Channels {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(String)]
    ChannelId { user_name: String },
    #[returns(Playlist)]
    Playlist {
        channel_id: String,
        playlist_name: String,
    },
    #[returns(Vec<Playlist>)]
    Playlists {
        channel_id: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(ChannelConractConfig)]
    Config {},
    #[returns(Vec<Asset>)]
    Assets {
        channel_id: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(Asset)]
    Asset {
        channel_id: String,
        publish_id: String,
    },
}
