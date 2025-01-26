use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::assets::level::Level;
use crate::exchange::Identifier;
use crate::model::ticker_info::TickerInfo;
use crate::shared::de::{de_str, de_u64_epoch_ms_as_datetime_utc};

/*----- */
// OrderBook Update L2
/*----- */
#[derive(Debug, Deserialize)]
pub struct AscendExBookUpdate {
    pub m: String,
    pub symbol: String,
    pub data: AscendExBookUpdateData,
}

#[derive(Debug, Deserialize)]
pub struct AscendExBookUpdateData {
    #[serde(deserialize_with = "de_u64_epoch_ms_as_datetime_utc")]
    pub ts: DateTime<Utc>,
    pub seqnum: u64,
    pub asks: Vec<Level>,
    pub bids: Vec<Level>,
}

impl Identifier<String> for AscendExBookUpdate {
    fn id(&self) -> String {
        self.symbol.clone()
    }
}

/*----- */
// OrderBook Snapshot
/*----- */
#[derive(Debug, Deserialize)]
pub struct AscendExOrderBookSnapshot {
    pub code: u64,
    pub data: AscendExOrderBookSnapshotData,
}

#[derive(Debug, Deserialize)]
pub struct AscendExOrderBookSnapshotData {
    pub m: String,
    pub symbol: String,
    pub data: AscendExOrderBookSnapshotTicks,
}

#[derive(Debug, Deserialize)]
pub struct AscendExOrderBookSnapshotTicks {
    #[serde(deserialize_with = "de_u64_epoch_ms_as_datetime_utc")]
    pub ts: DateTime<Utc>,
    pub seqnum: u64,
    pub asks: Vec<Level>,
    pub bids: Vec<Level>,
}

/*----- */
// Subscription Response
/*----- */
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum AscendExSubscriptionResponse {
    ConnectionSuccess {
        m: String,
        #[serde(rename = "type")]
        connection_type: String,
    },
    SubscriptionSuccess {
        m: String,
        id: String,
        ch: String,
        code: u64,
    },
    SubscriptionError {
        m: String,
        id: String,
        reason: String,
        code: u64,
        info: String,
    },
}

/*----- */
// Ticker info
/*----- */
#[derive(Debug, Deserialize)]
pub struct AscendExTickerInfo {
    pub code: u64,
    pub data: Vec<AscendExTickerInfoData>,
}

#[derive(Debug, Deserialize)]
pub struct AscendExTickerInfoData {
    pub symbol: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub domain: String,
    #[serde(rename = "tradingStartTime")]
    pub trading_start_time: u64,
    #[serde(rename = "collapseDecimals")]
    pub collapse_decimals: String,
    #[serde(rename = "minQty")]
    pub min_qty: String,
    #[serde(rename = "maxQty")]
    pub max_qty: String,
    #[serde(rename = "minNotional")]
    pub min_notional: String,
    #[serde(rename = "maxNotional")]
    pub max_notional: String,
    #[serde(rename = "statusCode")]
    pub status_code: String,
    #[serde(rename = "statusMessage")]
    pub status_message: String,
    #[serde(rename = "tickSize", deserialize_with = "de_str")]
    pub tick_size: f64,
    #[serde(rename = "useTick")]
    pub use_tick: bool,
    #[serde(rename = "lotSize")]
    pub lot_size: String,
    #[serde(rename = "useLot")]
    pub use_lot: bool,
    #[serde(rename = "commissionType")]
    pub commission_type: String,
    #[serde(rename = "commissionReserveRate")]
    pub commission_reserve_rate: String,
    #[serde(rename = "qtyScale")]
    pub qty_scale: i64,
    #[serde(rename = "priceScale")]
    pub price_scale: i64,
    #[serde(rename = "notionalScale")]
    pub notional_scale: i64,
}

impl From<AscendExTickerInfo> for TickerInfo {
    fn from(_value: AscendExTickerInfo) -> Self {
        unimplemented!()
    }
}
