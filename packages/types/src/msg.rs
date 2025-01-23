use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, Coin};

use crate::{
    asset::{Asset, AssetType, Playlist},
    channel::{ChannelCollaborator, ChannelDetails, ChannelMetadata},
    config::ChannelConractConfig,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Addr,
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
        /// Name of the channel
        channel_name: String,
        /// A description of the channel.
        description: Option<String>,
        /// The payment address of the channel owner.
        payment_address: Addr,
        /// (Optional) A list of collaborator addresses for the channel.
        collaborators: Option<Vec<String>>,
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
    /// Manages reserved usernames.
    /// Only callable by the protocol admin.
    /// Can set an address as a reserved username
    /// Can remove reserved usernames
    /// Can add reserved usernames
    ManageReservedUsernames {
        /// (Optional) A list of addresses to be set as reserved usernames.
        add_usernames: Option<Vec<ReservedUsername>>,
        /// (Optional) A list of addresses to be removed from reserved usernames.
        remove_usernames: Option<Vec<String>>,
    },
    TipCreator {
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
    #[returns(ChannelMetadata)]
    ChannelMetadata { channel_id: String },
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
    #[returns(Vec<ReservedUsername>)]
    ReservedUsernames {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}
