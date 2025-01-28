use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::assets::level::Level;
use crate::error::SocketError;
use crate::exchange::Identifier;
use crate::model::ticker_info::TickerInfo;
use crate::shared::de::de_u64_epoch_ns_as_datetime_utc;
use crate::streams::validator::Validator;

/*----- */
// OrderBook Update
/*----- */
#[derive(Debug, Deserialize)]
pub struct PhemexOrderBookUpdate {
    pub book: PhemexOrderBookUpdateData,
    pub depth: u32,
    pub sequence: u64,
    pub symbol: String,
    #[serde(deserialize_with = "de_u64_epoch_ns_as_datetime_utc")]
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "type")]
    pub message_type: String,
}

#[derive(Debug, Deserialize)]
pub struct PhemexOrderBookUpdateData {
    #[serde(deserialize_with = "de_levels_phemex")]
    pub asks: Vec<Level>,
    #[serde(deserialize_with = "de_levels_phemex")]
    pub bids: Vec<Level>,
}

impl Identifier<String> for PhemexOrderBookUpdate {
    fn id(&self) -> String {
        self.symbol.clone()
    }
}

fn de_levels_phemex<'de, D>(deserializer: D) -> Result<Vec<Level>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw_data: Vec<[u128; 2]> = Vec::deserialize(deserializer)?;

    Ok(raw_data
        .into_iter()
        .map(|entry| Level {
            price: (entry[0] as f64) / 100000000.0,
            size: (entry[1] as f64) / 100000000.0,
        })
        .collect())
}

/*----- */
// Subscription Response
/*----- */
#[derive(Debug, Deserialize)]
pub struct PhemexSubscriptionResponse {
    pub error: Option<serde_json::Value>,
    pub id: Option<u64>,
    pub result: Option<serde_json::Value>,
}

impl Validator for PhemexSubscriptionResponse {
    fn validate(self) -> Result<Self, SocketError> {
        if self.error.is_some() {
            Err(SocketError::Subscribe(format!(
                "received failure subscription response phemex. Error message: {}",
                self.error.unwrap() // Should never fail as we check self.error.is_some()
            )))
        } else {
            Ok(self)
        }
    }
}

/*----- */
// Ticker Info
/*----- */
#[derive(Debug, Deserialize)]
pub struct PhemexTickerInfo {
    pub code: i32,
    pub msg: String,
    pub data: PhemexTickerInfoData,
}

#[derive(Debug, Deserialize)]
pub struct PhemexTickerInfoData {
    pub currencies: Vec<PhemexTickerInfoCurrency>,
    pub products: Vec<PhemexTickerInfoProduct>,
    #[serde(rename = "perpProductsV2")]
    pub perp_products_v2: Vec<PhemexTickerInfoPerpProductV2>,
    #[serde(rename = "riskLimits")]
    pub risk_limits: Vec<PhemexTickerInfoRiskLimit>,
    pub leverages: Vec<PhemexTickerInfoLeverage>,
    #[serde(rename = "riskLimitsV2")]
    pub risk_limits_v2: Vec<PhemexTickerInfoRiskLimitV2>,
    #[serde(rename = "leveragesV2")]
    pub leverages_v2: Vec<PhemexTickerInfoLeverageV2>,
    #[serde(rename = "leverageMargins")]
    pub leverage_margins: Vec<PhemexTickerInfoLeverageMargin>,
    #[serde(rename = "ratioScale")]
    pub ratio_scale: i32,
    #[serde(rename = "md5Checksum")]
    pub md5_checksum: String,
}

#[derive(Debug, Deserialize)]
pub struct PhemexTickerInfoCurrency {
    pub currency: String,
    pub name: String,
    pub code: i32,
    #[serde(rename = "valueScale")]
    pub value_scale: i32,
    #[serde(rename = "minValueEv")]
    pub min_value_ev: i64,
    #[serde(rename = "maxValueEv")]
    pub max_value_ev: i64,
    #[serde(rename = "needAddrTag")]
    pub need_addr_tag: i32,
    pub status: String,
    #[serde(rename = "displayCurrency")]
    pub display_currency: String,
    #[serde(rename = "inAssetsDisplay")]
    pub in_assets_display: i32,
    pub perpetual: i32,
    #[serde(rename = "stableCoin")]
    pub stable_coin: i32,
    #[serde(rename = "assetsPrecision")]
    pub assets_precision: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct PhemexTickerInfoProduct {
    pub symbol: String,
    pub code: i32,
    #[serde(rename = "type")]
    pub product_type: String,
    #[serde(rename = "displaySymbol")]
    pub display_symbol: String,
    #[serde(rename = "indexSymbol")]
    pub index_symbol: Option<String>,
    #[serde(rename = "markSymbol")]
    pub mark_symbol: Option<String>,
    #[serde(rename = "fundingRateSymbol")]
    pub funding_rate_symbol: Option<String>,
    #[serde(rename = "fundingRate8hSymbol")]
    pub funding_rate_8h_symbol: Option<String>,
    #[serde(rename = "contractUnderlyingAssets")]
    pub contract_underlying_assets: Option<String>,
    #[serde(rename = "settleCurrency")]
    pub settle_currency: Option<String>,
    #[serde(rename = "quoteCurrency")]
    pub quote_currency: String,
    #[serde(rename = "contractSize")]
    pub contract_size: Option<f64>,
    #[serde(rename = "lotSize")]
    pub lot_size: Option<i32>,
    #[serde(rename = "tickSize")]
    pub tick_size: Option<f64>,
    #[serde(rename = "priceScale")]
    pub price_scale: i32,
    #[serde(rename = "ratioScale")]
    pub ratio_scale: i32,
    #[serde(rename = "pricePrecision")]
    pub price_precision: u32,
    #[serde(rename = "minPriceEp")]
    pub min_price_ep: Option<i64>,
    #[serde(rename = "maxPriceEp")]
    pub max_price_ep: Option<i64>,
    #[serde(rename = "maxOrderQty")]
    pub max_order_qty: Option<i64>,
    pub description: String,
    pub status: String,
    #[serde(rename = "tipOrderQty")]
    pub tip_order_qty: i64,
    #[serde(rename = "listTime")]
    pub list_time: i64,
    #[serde(rename = "majorSymbol")]
    pub major_symbol: Option<bool>,
    #[serde(rename = "defaultLeverage")]
    pub default_leverage: Option<String>,
    #[serde(rename = "fundingInterval")]
    pub funding_interval: Option<i32>,
    #[serde(rename = "maxLeverage")]
    pub max_leverage: Option<i32>,
    #[serde(rename = "leverageMargin")]
    pub leverage_margin: Option<i32>,
    pub leverage: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct PhemexTickerInfoPerpProductV2 {
    pub symbol: String,
    pub code: i32,
    #[serde(rename = "type")]
    pub product_type: String,
    #[serde(rename = "displaySymbol")]
    pub display_symbol: String,
    #[serde(rename = "indexSymbol")]
    pub index_symbol: String,
    #[serde(rename = "markSymbol")]
    pub mark_symbol: String,
    #[serde(rename = "fundingRateSymbol")]
    pub funding_rate_symbol: String,
    #[serde(rename = "fundingRate8hSymbol")]
    pub funding_rate_8h_symbol: String,
    #[serde(rename = "contractUnderlyingAssets")]
    pub contract_underlying_assets: String,
    #[serde(rename = "settleCurrency")]
    pub settle_currency: String,
    #[serde(rename = "quoteCurrency")]
    pub quote_currency: String,
    #[serde(rename = "tickSize")]
    pub tick_size: String,
    #[serde(rename = "priceScale")]
    pub price_scale: i32,
    #[serde(rename = "ratioScale")]
    pub ratio_scale: i32,
    #[serde(rename = "pricePrecision")]
    pub price_precision: i32,
    #[serde(rename = "baseCurrency")]
    pub base_currency: String,
    pub description: String,
    pub status: String,
    #[serde(rename = "tipOrderQty")]
    pub tip_order_qty: i64,
    #[serde(rename = "listTime")]
    pub list_time: i64,
    #[serde(rename = "majorSymbol")]
    pub major_symbol: bool,
    #[serde(rename = "defaultLeverage")]
    pub default_leverage: String,
    #[serde(rename = "fundingInterval")]
    pub funding_interval: i32,
    #[serde(rename = "maxLeverage")]
    pub max_leverage: i32,
    #[serde(rename = "leverageMargin")]
    pub leverage_margin: i32,
    #[serde(rename = "maxOrderQtyRq")]
    pub max_order_qty_rq: String,
    #[serde(rename = "maxPriceRp")]
    pub max_price_rp: String,
    #[serde(rename = "minOrderValueRv")]
    pub min_order_value_rv: String,
    #[serde(rename = "minPriceRp")]
    pub min_price_rp: String,
    #[serde(rename = "qtyPrecision")]
    pub qty_precision: i32,
    #[serde(rename = "qtyStepSize")]
    pub qty_step_size: String,
    #[serde(rename = "tipOrderQtyRq")]
    pub tip_order_qty_rq: String,
    #[serde(rename = "maxOpenPosLeverage")]
    pub max_open_pos_leverage: f64,
}

#[derive(Debug, Deserialize)]
pub struct PhemexTickerInfoRiskLimit {
    pub symbol: String,
    pub steps: String,
    #[serde(rename = "riskLimits")]
    pub risk_limits: Vec<PhemexTickerInfoRiskLimitItem>,
}

#[derive(Debug, Deserialize)]
pub struct PhemexTickerInfoRiskLimitItem {
    pub limit: i32,
    #[serde(rename = "initialMargin")]
    pub initial_margin: String,
    #[serde(rename = "initialMarginEr")]
    pub initial_margin_er: i32,
    #[serde(rename = "maintenanceMargin")]
    pub maintenance_margin: String,
    #[serde(rename = "maintenanceMarginEr")]
    pub maintenance_margin_er: i32,
}

#[derive(Debug, Deserialize)]
pub struct PhemexTickerInfoLeverage {
    #[serde(rename = "initialMargin")]
    pub initial_margin: String,
    #[serde(rename = "initialMarginEr")]
    pub initial_margin_er: i32,
    pub options: Vec<f64>,
}

#[derive(Debug, Deserialize)]
pub struct PhemexTickerInfoRiskLimitV2 {
    pub symbol: String,
    pub steps: String,
    #[serde(rename = "riskLimits")]
    pub risk_limits: Vec<PhemexTickerInfoRiskLimitItemV2>,
}

#[derive(Debug, Deserialize)]
pub struct PhemexTickerInfoRiskLimitItemV2 {
    pub limit: i32,
    #[serde(rename = "initialMarginRr")]
    pub initial_margin_rr: String,
    #[serde(rename = "maintenanceMarginRr")]
    pub maintenance_margin_rr: String,
}

#[derive(Debug, Deserialize)]
pub struct PhemexTickerInfoLeverageV2 {
    pub options: Vec<f64>,
    #[serde(rename = "initialMarginRr")]
    pub initial_margin_rr: String,
}

#[derive(Debug, Deserialize)]
pub struct PhemexTickerInfoLeverageMargin {
    pub index_id: i32,
    pub items: Vec<PhemexTickerInfoLeverageMarginItem>,
}

#[derive(Debug, Deserialize)]
pub struct PhemexTickerInfoLeverageMarginItem {
    #[serde(rename = "notionalValueRv")]
    pub notional_value_rv: i32,
    #[serde(rename = "maxLeverage")]
    pub max_leverage: f64,
    #[serde(rename = "maintenanceMarginRateRr")]
    pub maintenance_margin_rate_rr: String,
    #[serde(rename = "maintenanceAmountRv")]
    pub maintenance_amount_rv: String,
}

impl From<PhemexTickerInfo> for TickerInfo {
    fn from(_value: PhemexTickerInfo) -> Self {
        unimplemented!()
    }
}
