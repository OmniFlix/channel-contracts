use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, Coin, Decimal};

use crate::{
    asset::{Asset, AssetKey, AssetMetadata, AssetSource, Flag, Playlist},
    channel::{ChannelCollaborator, ChannelDetails, ChannelMetadata},
    config::ChannelConractConfig,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub protocol_admin: Addr,
    pub fee_collector: Addr,
    pub channels_collection_details: ChannelsCollectionDetails,
    pub channel_token_details: ChannelTokenDetails,
    pub channel_creation_fee: Vec<Coin>,
    pub accepted_tip_denoms: Vec<String>,
    pub reserved_usernames: Vec<ReservedUsername>,
}

#[cw_serde]
pub struct ChannelsCollectionDetails {
    pub collection_id: String,
    pub collection_name: String,
    pub collection_symbol: String,
    pub description: String,
    pub preview_uri: String,
    pub schema: String,
    pub uri: String,
    pub uri_hash: String,
    pub data: String,
}

#[cw_serde]
pub struct ChannelTokenDetails {
    pub description: String,
    pub media_uri: String,
    pub preview_uri: String,
    pub uri_hash: String,
    pub transferable: bool,
    pub extensible: bool,
    pub nsfw: bool,
    pub royalty_share: String,
}

#[cw_serde]
pub struct ReservedUsername {
    pub username: String,
    pub address: Option<Addr>,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Updates the configuration of the contract, including the channel creation fee,
    /// protocol admin, and fee collector. Only callable by the protocol admin.
    AdminSetConfig {
        /// (Optional) The new channel creation fee.
        channel_creation_fee: Option<Vec<Coin>>,
        /// (Optional) The new admin address.
        protocol_admin: Option<String>,
        /// (Optional) The new fee collector address.
        fee_collector: Option<String>,
    },
    /// Removes assets from the contract.
    /// Only callable by the protocol admin.    
    AdminRemoveAssets {
        /// The keys of the assets to be removed.
        asset_keys: Vec<AssetKey>,
        /// Removes all flags from the assets if set to true.
        refresh_flags: Option<bool>,
    },
    /// Manages reserved usernames.
    /// Only callable by the protocol admin.
    /// Can set an address as a reserved username
    /// Can remove reserved usernames
    /// Can add reserved usernames
    AdminManageReservedUsernames {
        /// (Optional) A list of addresses to be set as reserved usernames.
        add_usernames: Option<Vec<ReservedUsername>>,
        /// (Optional) A list of addresses to be removed from reserved usernames.
        remove_usernames: Option<Vec<String>>,
    },
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
    AssetPublish {
        /// The source of the asset.
        asset_source: AssetSource,
        /// A salt value used for unique identification.
        salt: Binary,
        /// The ID of the channel where the asset is published.
        channel_id: String,
        /// (Optional) The name of the playlist where the asset is added.
        playlist_id: Option<String>,
        /// A flag indicating if the asset is visible to the public.
        is_visible: bool,
        /// The metadata of the asset.
        metadata: AssetMetadata,
    },

    /// Unpublishes an asset from a channel. The publish ID and related asset details will
    /// be removed from the contract state. Only callable by the channel owner or a collaborator.
    AssetUnpublish {
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
        is_visible: Option<bool>,
        /// The new name of the asset.
        name: Option<String>,
        /// The new description of the asset.
        description: Option<String>,
        /// The new media URI of the asset.
        media_uri: Option<String>,
        /// The new thumbnail URI of the asset.
        thumbnail_uri: Option<String>,
    },
    AssetFlag {
        /// The ID of the channel where the asset is published.
        channel_id: String,
        /// The ID of the publish to be flagged.
        publish_id: String,
        /// The flag value.
        flag: Flag,
        /// Interactive video id. Only for indexing purposes. Not used for anything else.
        interactive_video_id: Option<String>,
    },
    /// Creates a new playlist in the specified channel.
    /// Only callable by the channel owner or a collaborator.
    PlaylistCreate {
        /// The name of the playlist.
        playlist_name: String,
        /// The ID of the channel where the playlist is created.
        channel_id: String,
        /// The salt value used for unique identification.
        salt: Binary,
    },

    /// Deletes an existing playlist from the channel.
    /// Only callable by the channel owner or a collaborator.
    PlaylistDelete {
        /// The ID of the playlist to be deleted.
        playlist_id: String,
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
        /// The ID of the playlist where the asset will be added.
        playlist_id: String,
    },

    /// Removes an asset from a playlist.
    /// Only callable by the channel owner or a collaborator.
    PlaylistRemoveAsset {
        /// The publish ID of the asset to be removed.
        publish_id: String,
        /// The ID of the channel where the playlist exists.
        channel_id: String,
        /// The ID of the playlist where the asset will be removed.
        playlist_id: String,
    },

    /// Refreshes a playlist by removing any assets that are no longer visible.
    /// Only callable by the channel owner or a collaborator.
    PlaylistRefresh {
        /// The ID of the channel where the playlist exists.
        channel_id: String,
        /// The ID of the playlist to be refreshed.
        playlist_id: String,
    },

    /// Creates a new channel. The contract will generate a channel ID and mint an NFT
    /// for the owner.
    /// The owner must pay the `channel_creation_fee` to create a channel.
    ChannelCreate {
        /// A salt value used for unique identification.
        salt: Binary,
        /// The user name of the channel owner.
        user_name: String,
        /// Name of the channel
        channel_name: String,
        /// A description of the channel.
        description: Option<String>,
        /// The payment address of the channel owner.
        payment_address: Addr,
        /// (Optional) Profile image of the channel
        profile_picture: Option<String>,
        /// (Optional) Banner image of the channel
        banner_picture: Option<String>,
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
        /// The new description of the channel.
        /// (Optional) The new description of the channel.
        description: Option<String>,
        /// (Optional) The new name of the channel.
        channel_name: Option<String>,
        /// (Optional) The new profile image of the channel.
        profile_picture: Option<String>,
        /// (Optional) The new banner image of the channel.
        banner_picture: Option<String>,
        /// (Optional) The new payment address of the channel.
        payment_address: Option<String>,
    },
    /// Tipping a channel
    ChannelTip {
        /// The ID of the channel to be tipped.
        channel_id: String,
        /// The amount of tokens to be tipped.
        amount: Coin,
        /// The asset id to be tipped. Only for indexing purposes. Not used for anything else.
        asset_id: Option<String>,
    },
    /// Adds a collaborator to a channel.
    /// Only callable by the channel owner.
    ChannelAddCollaborator {
        /// The ID of the channel to add the collaborator to.
        channel_id: String,
        /// The address of the collaborator to be added.
        collaborator_address: String,
        /// Collaborator details
        collaborator_details: ChannelCollaborator,
    },
    /// Removes a collaborator from a channel.
    /// Only callable by the channel owner.
    ChannelRemoveCollaborator {
        /// The ID of the channel to remove the collaborator from.
        channel_id: String,
        /// The address of the collaborator to be removed.
        collaborator_address: String,
    },

    /// Follow a channel
    ChannelFollow {
        /// The ID of the channel to follow.
        channel_id: String,
    },

    /// Unfollow a channel
    ChannelUnfollow {
        /// The ID of the channel to unfollow.
        channel_id: String,
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
    ChannelDetails { channel_id: String },

    #[returns(ChannelMetadata)]
    ChannelMetadata { channel_id: String },

    #[returns(ChannelResponse)]
    Channel { channel_id: String },

    #[returns(Vec<ChannelResponse>)]
    Channels {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(String)]
    ChannelId { user_name: String },

    /// Query a specific playlist by its ID.
    #[returns(Playlist)]
    Playlist {
        /// The ID of the channel where the playlist exists.
        channel_id: String,
        /// The ID of the playlist to query.
        playlist_id: String,
    },

    /// Query all playlists in a channel.
    #[returns(Vec<Playlist>)]
    Playlists {
        /// The ID of the channel to query playlists from.
        channel_id: String,
        /// Optional ID to start pagination after.
        start_after: Option<String>,
        /// Optional limit for pagination.
        limit: Option<u32>,
    },

    #[returns(ChannelConractConfig)]
    Config {},

    #[returns(Vec<AssetResponse>)]
    Assets {
        channel_id: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(AssetResponse)]
    Asset {
        channel_id: String,
        publish_id: String,
    },

    #[returns(Vec<ReservedUsername>)]
    ReservedUsernames {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(CollaboratorInfo)]
    GetChannelCollaborator {
        channel_id: String,
        collaborator_address: Addr,
    },

    #[returns(Vec<CollaboratorInfo>)]
    GetChannelCollaborators {
        channel_id: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(u64)]
    FollowersCount { channel_id: String },

    #[returns(Vec<String>)]
    Followers {
        channel_id: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
}
// Response for Channel query
#[cw_serde]
pub struct ChannelResponse {
    pub channel_id: String,
    pub user_name: String,
    pub onft_id: String,
    pub payment_address: String,
    pub channel_name: String,
    pub description: Option<String>,
    pub profile_picture: Option<String>,
    pub banner_picture: Option<String>,
    pub collaborators: Vec<CollaboratorInfo>,
    pub follower_count: u64,
}

#[cw_serde]
pub struct AssetResponse {
    pub asset: Asset,
    pub flags: Vec<FlagInfo>,
    pub metadata: AssetMetadata,
}
#[cw_serde]
pub struct CollaboratorInfo {
    pub address: String,
    pub role: String,
    pub share: Decimal,
}

// Create this new type to avoid tuples
#[cw_serde]
pub struct FlagInfo {
    pub flag: Flag,
    pub count: u64,
}
