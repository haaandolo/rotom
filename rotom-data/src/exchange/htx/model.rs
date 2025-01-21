use chrono::{DateTime, Utc};
use serde::Deserialize;

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
