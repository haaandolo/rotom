use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::model::event_trade::EventTrade;
use crate::shared::de::de_u64_epoch_ms_as_datetime_utc;
use crate::{
    assets::level::Level,
    error::SocketError,
    exchange::Identifier,
    model::{event_book_snapshot::EventOrderBookSnapshot, market_event::MarketEvent},
    shared::subscription_models::{ExchangeId, Instrument},
    streams::validator::Validator,
};

/*----- */
// Orderbook snapshot
/*----- */
#[derive(Debug, Default, Deserialize)]
pub struct HtxOrderBookSnapshot {
    pub ch: String,
    #[serde(deserialize_with = "de_u64_epoch_ms_as_datetime_utc")]
    pub ts: DateTime<Utc>,
    pub tick: HtxOrderBookSnapshotTick,
}

#[derive(Debug, Default, Deserialize)]
pub struct HtxOrderBookSnapshotTick {
    #[serde(rename = "seqNum")]
    pub seq_num: i64,
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
}

// todo: change from string to struct()
// have to split as ws data comes in like "market.glmrusdt.mbp.refresh.5" and we cant have this in stateless transformer
impl Identifier<String> for HtxOrderBookSnapshot {
    fn id(&self) -> String {
        self.ch.split('.').nth(1).unwrap_or_default().to_owned()
    }
}

impl From<(HtxOrderBookSnapshot, Instrument)> for MarketEvent<EventOrderBookSnapshot> {
    fn from((value, instrument): (HtxOrderBookSnapshot, Instrument)) -> Self {
        Self {
            exchange_time: value.ts,
            received_time: Utc::now(),
            exchange: ExchangeId::HtxSpot,
            instrument,
            event_data: EventOrderBookSnapshot {
                bids: value.tick.bids,
                asks: value.tick.asks,
            },
        }
    }
}

/*----- */
// Trade data
/*----- */
#[derive(Debug, Default, Deserialize)]
pub struct HtxTrade {
    pub ch: String,
    pub ts: u64,
    pub tick: HtxTradeData,
}

#[derive(Debug, Default, Deserialize)]
pub struct HtxTradeData {
    pub id: u64,
    #[serde(deserialize_with = "de_u64_epoch_ms_as_datetime_utc")]
    pub ts: DateTime<Utc>,
    pub data: Vec<HtxTradeTick>,
}

#[derive(Debug, Default, Deserialize)]
pub struct HtxTradeTick {
    pub id: u128,
    pub ts: u64,
    #[serde(rename = "tradeId")]
    pub trade_id: u64,
    pub amount: f64,
    pub price: f64,
    #[serde(deserialize_with = "de_buyer_is_maker_htx")]
    pub direction: bool,
}

impl Identifier<String> for HtxTrade {
    fn id(&self) -> String {
        self.ch.split('.').nth(1).unwrap_or_default().to_owned()
    }
}

impl From<(HtxTrade, Instrument)> for MarketEvent<Vec<EventTrade>> {
    fn from((event, instrument): (HtxTrade, Instrument)) -> Self {
        Self {
            exchange_time: event.tick.ts,
            received_time: Utc::now(),
            exchange: ExchangeId::HtxSpot,
            instrument,
            event_data: event
                .tick
                .data
                .iter()
                .map(|trade_data| {
                    EventTrade::new(
                        Level::new(trade_data.price, trade_data.amount),
                        trade_data.direction,
                    )
                })
                .collect::<Vec<EventTrade>>(),
        }
    }
}

pub fn de_buyer_is_maker_htx<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    <&str as Deserialize>::deserialize(deserializer).map(|buyer_is_maker| buyer_is_maker == "buy")
}

/*----- */
// Subscription response
/*----- */
#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum HtxSubscriptionResponse {
    Success {
        id: String,
        status: String,
        subbed: String,
        ts: u64,
    },
    Error {
        status: String,
        ts: u64,
        id: String,
        #[serde(rename = "err-code")]
        err_code: String,
        #[serde(rename = "err-msg")]
        err_msg: String,
    },
}

impl Validator for HtxSubscriptionResponse {
    fn validate(self) -> Result<Self, SocketError> {
        match &self {
            HtxSubscriptionResponse::Success { .. } => Ok(self),
            HtxSubscriptionResponse::Error {
                status,
                ts,
                id,
                err_code,
                err_msg,
            } => Err(SocketError::Subscribe(format!(
                "Error while subscribing to htx spot, status: {}, ts: {}, id: {}, err_code: {}, err_msg: {}",
                status,
                ts,
                id,
                err_code,
                err_msg,
            )))
        }
    }
}

/*----- */
// Wallet Info
/*----- */
#[derive(Debug, Deserialize)]
pub struct HtxNetworkInfo {
    pub data: Vec<HtxNetworkInfoData>,
    pub full: i64,
    pub status: String,
    pub ts: String,
}

#[derive(Debug, Deserialize)]
pub struct HtxNetworkInfoData {
    pub ac: String,
    pub adt: bool,
    pub ao: bool,
    pub awt: bool,
    pub ca: String,
    pub cct: i64,
    pub chain: String,
    pub code: String,
    pub ct: String,
    pub currency: String,
    pub de: bool,
    pub default: i64,
    #[serde(rename = "deposit-desc")]
    #[serde(default)]
    pub deposit_desc: String,
    pub dma: String,
    pub dn: String,
    pub fc: i64,
    #[serde(rename = "fn")]
    pub fn_name: String, // using fn_name since 'fn' is a reserved keyword in Rust
    pub ft: String,
    pub sc: i64,
    pub sda: Option<String>,
    #[serde(rename = "suspend-deposit-desc")]
    pub suspend_deposit_desc: Option<String>,
    #[serde(rename = "suspend-withdraw-desc")]
    pub suspend_withdraw_desc: Option<String>,
    pub swa: Option<String>,
    pub v: bool,
    pub we: bool,
    #[serde(rename = "withdraw-desc")]
    #[serde(default)]
    pub withdraw_desc: String,
    pub wma: String,
    pub wp: i64,
}
