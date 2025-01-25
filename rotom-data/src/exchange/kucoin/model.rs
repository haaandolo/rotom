use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::assets::level::Level;
use crate::error::SocketError;
use crate::exchange::Identifier;
use crate::model::event_book_snapshot::EventOrderBookSnapshot;
use crate::model::market_event::MarketEvent;
use crate::shared::de::de_u64_epoch_ms_as_datetime_utc;
use crate::shared::subscription_models::{ExchangeId, Instrument};
use crate::streams::validator::Validator;

/*----- */
// Kucoin Ws URL response
/*----- */
#[derive(Debug, Deserialize)]
pub struct KuCoinWsUrl {
    pub code: String,
    pub data: KuCoinWsUrlData,
}

#[derive(Debug, Deserialize)]
pub struct KuCoinWsUrlData {
    pub token: String,
    #[serde(rename = "instanceServers")]
    pub instance_servers: Vec<KuCoinInstanceServer>,
}

#[derive(Debug, Deserialize)]
pub struct KuCoinInstanceServer {
    pub endpoint: String,
    pub encrypt: bool,
    pub protocol: String,
    #[serde(rename = "pingInterval")]
    pub ping_interval: u32,
    #[serde(rename = "pingTimeout")]
    pub ping_timeout: u32,
}

/*----- */
// OrderBook Snapshot
/*----- */
#[derive(Debug, Deserialize, Default)]
pub struct KuCoinOrderBookSnapshot {
    pub topic: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub subject: String,
    pub data: KuCoinOrderBookSnapshotData,
}

#[derive(Debug, Deserialize, Default)]
pub struct KuCoinOrderBookSnapshotData {
    pub asks: Vec<Level>,
    pub bids: Vec<Level>,
    #[serde(deserialize_with = "de_u64_epoch_ms_as_datetime_utc")]
    pub timestamp: DateTime<Utc>,
}

impl Identifier<String> for KuCoinOrderBookSnapshot {
    fn id(&self) -> String {
        self.topic.split(':').last().unwrap().to_owned()
    }
}

impl From<(KuCoinOrderBookSnapshot, Instrument)> for MarketEvent<EventOrderBookSnapshot> {
    fn from((value, instrument): (KuCoinOrderBookSnapshot, Instrument)) -> Self {
        Self {
            exchange_time: value.data.timestamp,
            received_time: Utc::now(),
            exchange: ExchangeId::KuCoinSpot,
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
#[derive(Debug, Deserialize)]
pub struct KuCoinSubscriptionResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub response_type: String,
    #[serde(default)]
    pub code: u64,
    #[serde(default)]
    pub data: String,
}

impl Validator for KuCoinSubscriptionResponse {
    fn validate(self) -> Result<Self, SocketError> {
        if self.response_type == *"error" {
            Err(SocketError::Subscribe(
                format!("received failure subscription response for kucoin. Error code: {}. Error message {}", self.code, self.data),
            ))
        } else {
            Ok(self)
        }
    }
}
