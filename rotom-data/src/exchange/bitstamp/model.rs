use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::{assets::level::Level, shared::de::de_str_u64_epoch_ms_as_datetime_utc};

#[derive(Debug, Deserialize)]
pub struct BitstampOrderBookSnapshot {
    pub channel: String,
    pub event: String,
    pub data: BitstampOrderBookSnapshotData,
}

#[derive(Debug, Deserialize)]
pub struct BitstampOrderBookSnapshotData {
    #[serde(deserialize_with = "de_str_u64_epoch_ms_as_datetime_utc")]
    pub timestamp: DateTime<Utc>,
    pub microtimestamp: String,
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
}
