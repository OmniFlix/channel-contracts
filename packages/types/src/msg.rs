use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, Coin};

use crate::{
    asset::{Asset, AssetKey, AssetSource, Flag, Playlist},
    channel::{ChannelCollaborator, ChannelDetails, ChannelMetadata},
    config::ChannelConractConfig,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub protocol_admin: Addr,
    pub fee_collector: Addr,
    pub channels_collection_id: String,
    pub channels_collection_name: String,
    pub channels_collection_symbol: String,
    pub channel_creation_fee: Vec<Coin>,
    pub accepted_tip_denoms: Vec<String>,
    pub reserved_usernames: Vec<ReservedUsername>,
}

#[cw_serde]
pub struct ReservedUsername {
    pub username: String,
    pub address: Option<String>,
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
        /// The flags and the count of flag limits to be removed.
        flags: Option<Vec<(Flag, u64)>>,
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
        asset_source: AssetSource,
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
        is_visible: bool,
    },
    AssetFlag {
        /// The ID of the channel where the asset is published.
        channel_id: String,
        /// The ID of the publish to be flagged.
        publish_id: String,
        /// The flag value.
        flag: Flag,
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
    #[returns(IsPausedResponse)]
    IsPaused {},
    #[returns(PausersResponse)]
    Pausers {},
    #[returns(ChannelDetailsResponse)]
    ChannelDetails { channel_id: String },
    #[returns(ChannelMetadataResponse)]
    ChannelMetadata { channel_id: String },
    #[returns(ChannelResponse)]
    Channel { channel_id: String },
    #[returns(ChannelsResponse)]
    Channels {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(ChannelIdResponse)]
    ChannelId { user_name: String },
    #[returns(PlaylistResponse)]
    Playlist {
        channel_id: String,
        playlist_name: String,
    },
    #[returns(PlaylistsResponse)]
    Playlists {
        channel_id: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(ConfigResponse)]
    Config {},
    #[returns(AssetsResponse)]
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
    #[returns(ReservedUsernamesResponse)]
    ReservedUsernames {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(GetChannelCollaboratorResponse)]
    GetChannelCollaborator {
        channel_id: String,
        collaborator_address: Addr,
    },
    #[returns(GetChannelCollaboratorsResponse)]
    GetChannelCollaborators {
        channel_id: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(FollowersCountResponse)]
    FollowersCount { channel_id: String },
    #[returns(FollowersResponse)]
    Followers {
        channel_id: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

// Response for IsPaused query
#[cw_serde]
pub struct IsPausedResponse {
    pub is_paused: bool,
}

// Response for Pausers query
#[cw_serde]
pub struct PausersResponse {
    pub pausers: Vec<String>,
}

// Response for ChannelDetails query
#[cw_serde]
pub struct ChannelDetailsResponse {
    pub details: ChannelDetails,
}

// Response for ChannelMetadata query
#[cw_serde]
pub struct ChannelMetadataResponse {
    pub metadata: ChannelMetadata,
}

// Response for Channels query
#[cw_serde]
pub struct ChannelsResponse {
    pub channels: Vec<ChannelResponse>,
}

// Response for ChannelId query
#[cw_serde]
pub struct ChannelIdResponse {
    pub channel_id: String,
}

// Response for Playlist query
#[cw_serde]
pub struct PlaylistResponse {
    pub playlist: Playlist,
}

// Response for Playlists query
#[cw_serde]
pub struct PlaylistsResponse {
    pub playlists: Vec<Playlist>,
}

// Response for Config query
#[cw_serde]
pub struct ConfigResponse {
    pub config: ChannelConractConfig,
}

// Response for Assets query
#[cw_serde]
pub struct AssetsResponse {
    pub assets: Vec<AssetResponse>,
}

// Response for Asset query
#[cw_serde]
pub struct AssetResponse {
    pub asset: Asset,
    pub flags: Vec<(Flag, u64)>,
}

// Response for ReservedUsernames query
#[cw_serde]
pub struct ReservedUsernamesResponse {
    pub reserved_usernames: Vec<ReservedUsername>,
}

// Response for GetChannelCollaborator query
#[cw_serde]
pub struct GetChannelCollaboratorResponse {
    pub collaborator: ChannelCollaborator,
}

// Response for GetChannelCollaborators query
#[cw_serde]
pub struct GetChannelCollaboratorsResponse {
    pub collaborators: Vec<(Addr, ChannelCollaborator)>,
}

// Response for FollowersCount query
#[cw_serde]
pub struct FollowersCountResponse {
    pub count: u64,
}

// Response for Followers query
#[cw_serde]
pub struct FollowersResponse {
    pub followers: Vec<Addr>,
}

// Response for Channel query
#[cw_serde]
pub struct ChannelResponse {
    pub channel_details: ChannelDetails,
    pub channel_metadata: ChannelMetadata,
    pub channel_collaborators: Vec<(String, ChannelCollaborator)>,
}
