use rotom_data::shared::de::de_str;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RateLimit {
    #[serde(alias = "rateLimitType")]
    pub rate_limit_type: String,
    pub interval: String,
    #[serde(alias = "intervalNum")]
    pub interval_num: u32,
    pub limit: u32,
    pub count: u32,
}

#[derive(Debug, Deserialize)]
pub struct BinanceFill {
    #[serde(deserialize_with = "de_str")]
    pub price: f64,
    #[serde(deserialize_with = "de_str")]
    pub qty: f64,
    #[serde(deserialize_with = "de_str")]
    pub commission: f64,
    #[serde(alias = "commissionAsset")]
    pub commission_asset: String,
    #[serde(alias = "tradeId")]
    pub trade_id: u64,
}
