use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::{
    assets::level::Level,
    error::SocketError,
    exchange::Identifier,
    model::{event_book_snapshot::EventOrderBookSnapshot, market_event::MarketEvent},
    shared::{
        de::de_u64_epoch_ms_as_datetime_utc,
        subscription_models::{ExchangeId, Instrument},
    },
    streams::validator::Validator,
};

/*----- */
// OrderBook Snapshot
/*----- */
#[derive(Debug, Default, Deserialize)]
pub struct WooxOrderBookSnapshot {
    pub topic: String,
    #[serde(deserialize_with = "de_u64_epoch_ms_as_datetime_utc")]
    pub ts: DateTime<Utc>,
    pub data: WooxOrderBookSnapshotData,
}

#[derive(Debug, Default, Deserialize)]
pub struct WooxOrderBookSnapshotData {
    pub symbol: String,
    pub asks: Vec<Level>,
    pub bids: Vec<Level>,
}

impl Identifier<String> for WooxOrderBookSnapshot {
    fn id(&self) -> String {
        self.data.symbol.clone()
    }
}

impl From<(WooxOrderBookSnapshot, Instrument)> for MarketEvent<EventOrderBookSnapshot> {
    fn from((value, instrument): (WooxOrderBookSnapshot, Instrument)) -> Self {
        Self {
            exchange_time: value.ts,
            received_time: Utc::now(),
            exchange: ExchangeId::HtxSpot,
            instrument,
            event_data: EventOrderBookSnapshot {
                bids: value.data.bids,
                asks: value.data.asks,
            },
        }
    }
}

/*----- */
// Subscription Response
/*----- */
#[derive(Debug, Deserialize, PartialEq)]
pub struct WooxSubscriptionResponse {
    id: String,
    event: String,
    success: bool,
    ts: u64,
    #[serde(default)]
    data: String,
    #[serde(default)]
    #[serde(rename = "errorMsg")]
    error_msg: String,
}

impl Validator for WooxSubscriptionResponse {
    fn validate(self) -> Result<Self, SocketError> {
        if self.success {
            Ok(self)
        } else {
            Err(SocketError::Subscribe(
                "received failure subscription response for WooxSpot".to_owned(),
            ))
        }
    }
}
