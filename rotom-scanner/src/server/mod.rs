pub mod handlers;
pub mod server_channels;

use rotom_data::shared::subscription_models::{ExchangeId, Instrument};
use serde::Serialize;

use crate::spot_scanner::scanner::{SpreadHistoryResponse, SpreadResponse};

#[derive(Debug, Serialize)]
pub enum SpotArbScannerHttpRequests {
    GetTopSpreads,
    GetSpreadHistory((ExchangeId, ExchangeId, Instrument)),
    GetWsConnectionStatus,
}

#[derive(Debug, Serialize)]
pub enum SpotArbScannerHttpResponse {
    GetTopSpreads(Vec<SpreadResponse>),
    GetSpreadHistory(Box<SpreadHistoryResponse>),
    GetWsConnectionStatus {
        snapshot: u32,
        trade: u32,
    },
    CouldNotFindSpreadHistory {
        base_exchange: ExchangeId,
        quote_exchange: ExchangeId,
        instrument: Instrument,
    },
}
