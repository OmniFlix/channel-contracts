use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct Playlist {
    pub playlist_name: String,
    pub assets: Vec<Asset>,
}

#[cw_serde]
pub struct Asset {
    pub channel_id: String,
    pub publish_id: String,
    pub collection_id: String,
    pub onft_id: String,
    pub is_visible: bool,
}
