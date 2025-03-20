use cosmwasm_std::{Addr, Binary};
use omniflix_channel_types::{
    asset::{AssetMetadata, AssetSource},
    msg::{
        ChannelTokenDetails, ChannelsCollectionDetails, ExecuteMsg, InstantiateMsg,
        ReservedUsername,
    },
};

pub fn get_channel_instantiate_msg(admin: Addr) -> InstantiateMsg {
    InstantiateMsg {
        channel_creation_fee: vec![],
        fee_collector: admin.clone(),
        protocol_admin: admin.clone(),
        accepted_tip_denoms: vec!["uflix".to_string()],
        channel_token_details: ChannelTokenDetails {
            media_uri: "https://example.com/media.png".to_string(),
            preview_uri: "https://example.com/preview.png".to_string(),
            description: "Channel token details".to_string(),
            uri_hash: "".to_string(),
            transferable: true,
            extensible: true,
            nsfw: false,
            royalty_share: "0".to_string(),
        },
        channels_collection_details: ChannelsCollectionDetails {
            collection_id: "Channels".to_string(),
            collection_name: "Channels".to_string(),
            collection_symbol: "CH".to_string(),
            description: "Channels collection".to_string(),
            preview_uri: "https://example.com/preview.png".to_string(),
            uri: "https://example.com/uri".to_string(),
            schema: "https://example.com/schema".to_string(),
            uri_hash: "".to_string(),
            data: "".to_string(),
        },
        reserved_usernames: vec![ReservedUsername {
            username: "reserved".to_string(),
            address: None,
        }],
    }
}

pub struct CreateChannelMsgBuilder {
    salt: Binary,
    user_name: String,
    description: String,
    channel_name: String,
    banner_picture: Option<String>,
    profile_picture: Option<String>,
    payment_address: Addr,
}

impl CreateChannelMsgBuilder {
    pub fn new(user_name: &str, payment_address: Addr) -> Self {
        Self {
            salt: Binary::from("salt".as_bytes()),
            user_name: user_name.to_string(),
            description: "Default description".to_string(),
            channel_name: user_name.to_string(), // Default to the same as user_name
            banner_picture: None,
            profile_picture: None,
            payment_address,
        }
    }

    pub fn description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn channel_name(mut self, channel_name: String) -> Self {
        self.channel_name = channel_name;
        self
    }

    pub fn banner_picture(mut self, banner_picture: String) -> Self {
        self.banner_picture = Some(banner_picture);
        self
    }

    pub fn profile_picture(mut self, profile_picture: String) -> Self {
        self.profile_picture = Some(profile_picture);
        self
    }

    pub fn salt(mut self, salt: Binary) -> Self {
        self.salt = salt;
        self
    }

    pub fn build(self) -> ExecuteMsg {
        ExecuteMsg::ChannelCreate {
            salt: self.salt,
            user_name: self.user_name,
            description: Some(self.description),
            banner_picture: self.banner_picture,
            profile_picture: self.profile_picture,
            channel_name: self.channel_name,
            payment_address: self.payment_address,
        }
    }
}

pub struct AssetPublishMsgBuilder {
    asset_source: AssetSource,
    salt: Binary,
    channel_id: String,
    playlist_name: Option<String>,
    is_visible: bool,
    name: String,
    description: String,
    media_uri: String,
}

impl AssetPublishMsgBuilder {
    pub fn new(channel_id: String) -> Self {
        Self {
            asset_source: AssetSource::OffChain {},
            salt: Binary::from("salt".as_bytes()),
            channel_id: channel_id,
            playlist_name: None,
            is_visible: true,
            name: "validassetname".to_string(),
            description: "validassetdescription".to_string(),
            media_uri: "https://example.com/media.png".to_string(),
        }
    }

    pub fn asset_source(mut self, asset_source: AssetSource) -> Self {
        self.asset_source = asset_source;
        self
    }

    pub fn playlist_name(mut self, playlist_name: String) -> Self {
        self.playlist_name = Some(playlist_name);
        self
    }

    pub fn is_visible(mut self, is_visible: bool) -> Self {
        self.is_visible = is_visible;
        self
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn media_uri(mut self, media_uri: String) -> Self {
        self.media_uri = media_uri;
        self
    }

    pub fn build(self) -> ExecuteMsg {
        ExecuteMsg::AssetPublish {
            asset_source: self.asset_source,
            salt: self.salt,
            channel_id: self.channel_id,
            playlist_name: self.playlist_name,
            is_visible: self.is_visible,
            metadata: AssetMetadata {
                name: self.name,
                description: self.description,
                media_uri: self.media_uri,
            },
        }
    }
}
