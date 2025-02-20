use cw_storage_plus::Item;
use omniflix_channel_types::config::{AuthDetails, ChannelConractConfig};

pub type ChannelsCollectionId = String;

// Define storage items
pub const CONFIG: Item<ChannelConractConfig> = Item::new("chcfg");
pub const AUTH_DETAILS: Item<AuthDetails> = Item::new("auth_details");
