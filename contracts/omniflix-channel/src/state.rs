use cw_storage_plus::Item;
use omniflix_channel_types::{
    config::{AuthDetails, ChannelConractConfig},
    msg::ChannelTokenDetails,
};

pub type ChannelsCollectionId = String;

// Define storage items
pub const CONFIG: Item<ChannelConractConfig> = Item::new("chcfg");
pub const AUTH_DETAILS: Item<AuthDetails> = Item::new("auth");
pub const CHANNEL_TOKEN_DETAILS: Item<ChannelTokenDetails> = Item::new("ch_tkn_details");
