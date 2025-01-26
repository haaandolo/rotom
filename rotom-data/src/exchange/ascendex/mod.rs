use async_trait::async_trait;
use model::{AscendExOrderBookSnapshot, AscendExTickerInfo};

use crate::{
    error::SocketError,
    shared::subscription_models::{ExchangeId, Instrument},
};

use super::PublicHttpConnector;

pub mod channel;
pub mod l2;
pub mod market;
pub mod model;

/*----- */
// Connector
/*----- */
#[derive(Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Clone)]
pub struct AscendExSpotPublicData;

/*----- */
// HttpConnector
/*----- */
pub const ASCENDEX_BASE_HTTP_URL: &str = "https://ascendex.com";

#[async_trait]
impl PublicHttpConnector for AscendExSpotPublicData {
    const ID: ExchangeId = ExchangeId::AscendExSpot;

    type BookSnapShot = AscendExOrderBookSnapshot;
    type ExchangeTickerInfo = AscendExTickerInfo;
    type NetworkInfo = serde_json::Value;

    async fn get_book_snapshot(instrument: Instrument) -> Result<Self::BookSnapShot, SocketError> {
        let request_path = "/api/pro/v1/depth";
        let snapshot_url = format!(
            "{}{}?symbol={}/{}",
            ASCENDEX_BASE_HTTP_URL,
            request_path,
            instrument.base.to_uppercase(),
            instrument.quote.to_uppercase()
        );

        reqwest::get(snapshot_url)
            .await
            .map_err(SocketError::Http)?
            .json::<Self::BookSnapShot>()
            .await
            .map_err(SocketError::Http)
    }

    async fn get_ticker_info(
        _instrument: Instrument,
    ) -> Result<Self::ExchangeTickerInfo, SocketError> {
        let request_path = "/api/pro/v1/cash/products";
        let ticker_info_url = format!("{}{}", ASCENDEX_BASE_HTTP_URL, request_path,);

        reqwest::get(ticker_info_url)
            .await
            .map_err(SocketError::Http)?
            .json::<Self::ExchangeTickerInfo>()
            .await
            .map_err(SocketError::Http)
    }

    async fn get_network_info() -> Result<Self::NetworkInfo, SocketError> {
        unimplemented!()
    }
}
