use ::serde::Deserialize;
use chrono::{DateTime, Utc};

use crate::assets::level::Level;
use crate::error::SocketError;
use crate::exchange::Identifier;
use crate::model::event_book_snapshot::EventOrderBookSnapshot;
use crate::model::market_event::MarketEvent;
use crate::shared::de::de_u64_epoch_ms_as_datetime_utc;
use crate::shared::subscription_models::{ExchangeId, Instrument};
use crate::streams::validator::Validator;

/*----- */
// OrderBook Snapshot
/*----- */
#[derive(Debug, Default, Deserialize)]
pub struct CoinExOrderBookSnapshot {
    pub data: CoinExOrderBookSnapshotData,
    pub id: Option<serde_json::Value>,
    pub method: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct CoinExOrderBookSnapshotData {
    pub depth: CoinExOrderBookSnapshotTick,
    #[serde(rename = "is_full")]
    pub is_full: bool,
    pub market: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct CoinExOrderBookSnapshotTick {
    pub asks: Vec<Level>,
    pub bids: Vec<Level>,
    pub checksum: i64,
    pub last: String,
    #[serde(
        rename = "updated_at",
        deserialize_with = "de_u64_epoch_ms_as_datetime_utc"
    )]
    pub updated_at: DateTime<Utc>,
}

impl Identifier<String> for CoinExOrderBookSnapshot {
    fn id(&self) -> String {
        self.data.market.clone()
    }
}

impl From<(CoinExOrderBookSnapshot, Instrument)> for MarketEvent<EventOrderBookSnapshot> {
    fn from((value, instrument): (CoinExOrderBookSnapshot, Instrument)) -> Self {
        Self {
            exchange_time: value.data.depth.updated_at,
            received_time: Utc::now(),
            exchange: ExchangeId::HtxSpot,
            instrument,
            event_data: EventOrderBookSnapshot {
                bids: value.data.depth.bids,
                asks: value.data.depth.asks,
            },
        }
    }
}

/*----- */
// Subscription Response
/*----- */
#[derive(Debug, Deserialize)]
pub struct CoinExSubscriptionResponse {
    pub id: u64,
    pub code: u64,
    pub message: String,
}

impl Validator for CoinExSubscriptionResponse {
    fn validate(self) -> Result<Self, SocketError> {
        if self.message == *"OK" {
            Ok(self)
        } else {
            Err(SocketError::Subscribe(format!(
                "received failure subscription response for CoinExSpot. Expected OK but got {}",
                self.message
            )))
        }
    }
}
