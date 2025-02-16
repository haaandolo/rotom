pub mod handlers;
pub mod server_channels;

use rotom_data::shared::subscription_models::{ExchangeId, Instrument};
use serde::Serialize;

use crate::spot_scanner::scanner::SpreadResponse;

#[derive(Debug, Serialize)]
pub enum SpotArbScannerHttpRequests {
    GetTopSpreads,
    GetSpreadHistory((ExchangeId, Instrument)),
}

#[derive(Debug, Serialize)]
pub enum SpotArbScannerHttpResponse {
    GetTopSpreads(Vec<SpreadResponse>),
    // GetSpreadHistory((ExchangeId, Instrument)),
}