use crate::{error::SocketError, shared::subscription_models::Instrument};

use super::requests::{book_snapshot::BinanceSpotSnapshot, ticker_info::BinanceSpotTickerInfo};

pub const HTTP_BOOK_L2_SNAPSHOT_URL_BINANCE_SPOT: &str = "https://api.binance.com/api/v3/depth";
pub const HTTP_TICKER_INFO_URL_BINANCE_SPOT: &str =
    "https://api.binance.us/api/v3/exchangeInfo?symbol=";

/*----- */
// Binance Public Data
/*----- */
#[derive(Debug)]
pub struct BinancePublicData;

impl BinancePublicData {
    pub async fn get_book_snapshot(
        instrument: &Instrument,
    ) -> Result<BinanceSpotSnapshot, SocketError> {
        let snapshot_url = format!(
            "{}?symbol={}{}&limit=100",
            HTTP_BOOK_L2_SNAPSHOT_URL_BINANCE_SPOT,
            instrument.base.to_uppercase(),
            instrument.quote.to_uppercase()
        );

        reqwest::get(snapshot_url)
            .await
            .map_err(SocketError::Http)?
            .json::<BinanceSpotSnapshot>()
            .await
            .map_err(SocketError::Http)
    }

    pub async fn get_ticker_info(
        instrument: &Instrument,
    ) -> Result<BinanceSpotTickerInfo, SocketError> {
        let ticker_info_url = format!(
            "{}{}{}",
            HTTP_TICKER_INFO_URL_BINANCE_SPOT,
            instrument.base.to_uppercase(),
            instrument.quote.to_uppercase()
        );

        reqwest::get(ticker_info_url)
            .await
            .map_err(SocketError::Http)?
            .json::<BinanceSpotTickerInfo>()
            .await
            .map_err(SocketError::Http)
    }
}
