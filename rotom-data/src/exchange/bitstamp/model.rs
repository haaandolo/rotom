use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::{
    assets::level::Level,
    error::SocketError,
    exchange::Identifier,
    model::{event_book_snapshot::EventOrderBookSnapshot, market_event::MarketEvent},
    shared::{
        de::de_str_u64_epoch_ms_as_datetime_utc,
        subscription_models::{ExchangeId, Instrument},
    },
    streams::validator::Validator,
};

/*----- */
// OrderBook Snapshot
/*----- */
#[derive(Debug, Default, Deserialize)]
pub struct BitstampOrderBookSnapshot {
    pub channel: String,
    pub event: String,
    pub data: BitstampOrderBookSnapshotData,
}

#[derive(Debug, Default, Deserialize)]
pub struct BitstampOrderBookSnapshotData {
    #[serde(deserialize_with = "de_str_u64_epoch_ms_as_datetime_utc")]
    pub timestamp: DateTime<Utc>,
    pub microtimestamp: String,
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
}

impl Identifier<String> for BitstampOrderBookSnapshot {
    fn id(&self) -> String {
        self.channel
            .split('_')
            .last()
            .unwrap_or_default()
            .to_owned()
    }
}

impl From<(BitstampOrderBookSnapshot, Instrument)> for MarketEvent<EventOrderBookSnapshot> {
    fn from((value, instrument): (BitstampOrderBookSnapshot, Instrument)) -> Self {
        Self {
            exchange_time: value.data.timestamp,
            received_time: Utc::now(),
            exchange: ExchangeId::BitstampSpot,
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
pub struct BitstampSubscriptionResponse {
    event: String,
    channel: String,
    #[serde(skip_deserializing)]
    data: serde_json::Value,
}

impl Validator for BitstampSubscriptionResponse {
    fn validate(self) -> Result<Self, SocketError> {
        if self.event == "bts:error" {
            Err(SocketError::Subscribe(
                "received failure subscription response for Bitstamp".to_owned(),
            ))
        } else {
            Ok(self)
        }
    }
}
