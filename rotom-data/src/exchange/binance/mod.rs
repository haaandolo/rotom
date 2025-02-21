pub mod channel;
pub mod l2;
pub mod market;
pub mod model;

use async_trait::async_trait;
use channel::BinanceChannel;
use chrono::Utc;
use hmac::{Hmac, Mac};
use l2::BinanceSpotBookUpdater;
use market::BinanceMarket;
use model::{
    BinanceAggTrade, BinanceNetworkInfo, BinanceSpotBookUpdate, BinanceSpotSnapshot,
    BinanceSpotTickerInfo, BinanceSubscriptionResponse, BinanceTrade,
};
use serde_json::json;
use sha2::Sha256;

use crate::{
    error::SocketError,
    exchange::{PublicHttpConnector, PublicStreamConnector, StreamSelector},
    model::{
        event_book::OrderBookL2,
        event_trade::{AggTrades, Trade},
    },
    protocols::ws::WsMessage,
    shared::subscription_models::{ExchangeId, ExchangeSubscription, Instrument},
    transformer::{book::MultiBookTransformer, stateless_transformer::StatelessTransformer},
};

const BINANCE_SPOT_WS_URL: &str = "wss://stream.binance.com:9443/ws";

/*----- */
// BinanceSpot connector
/*----- */
#[derive(Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Clone)]
pub struct BinanceSpotPublicData;

impl PublicStreamConnector for BinanceSpotPublicData {
    const ID: ExchangeId = ExchangeId::BinanceSpot;

    type SubscriptionResponse = BinanceSubscriptionResponse;
    type Channel = BinanceChannel;
    type Market = BinanceMarket;

    fn url() -> impl Into<String> {
        BINANCE_SPOT_WS_URL
    }

    fn requests(
        subscriptions: &[ExchangeSubscription<Self, Self::Channel, Self::Market>],
    ) -> Option<WsMessage> {
        let binance_subs = subscriptions
            .iter()
            .map(|s| format!("{}{}", s.market.as_ref().to_lowercase(), s.channel.as_ref()))
            .collect::<Vec<_>>();

        let binance_request = json!({
            "method": "SUBSCRIBE",
            "params": binance_subs,
            "id": 1
        });

        Some(WsMessage::Text(binance_request.to_string()))
    }

    fn expected_responses(
        _subscriptions: &[ExchangeSubscription<Self, Self::Channel, Self::Market>],
    ) -> usize {
        1
    }
}

/*----- */
// BinanceSpot HttpConnector
/*----- */
pub const BINANCE_BASE_HTTP_URL: &str = "https://api.binance.com";
pub const BINANCE_BASE_HTTP_URL2: &str = "https://api.binance.us";

#[async_trait]
impl PublicHttpConnector for BinanceSpotPublicData {
    const ID: ExchangeId = ExchangeId::BinanceSpot;

    type BookSnapShot = BinanceSpotSnapshot;
    type ExchangeTickerInfo = BinanceSpotTickerInfo;
    type NetworkInfo = Vec<BinanceNetworkInfo>;

    async fn get_book_snapshot(instrument: Instrument) -> Result<Self::BookSnapShot, SocketError> {
        let request_path = "/api/v3/depth";
        let snapshot_url = format!(
            "{}{}?symbol={}{}&limit=100",
            BINANCE_BASE_HTTP_URL,
            request_path,
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

    async fn get_ticker_info(
        instrument: Instrument,
    ) -> Result<Self::ExchangeTickerInfo, SocketError> {
        let request_path = "/api/v3/exchangeInfo?symbol=";
        let ticker_info_url = format!(
            "{}{}{}{}",
            BINANCE_BASE_HTTP_URL2,
            request_path,
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

    async fn get_network_info(
        _instruments: Vec<Instrument>,
    ) -> Result<Self::NetworkInfo, SocketError> {
        // Import secrets, key etc
        let secret = env!("BINANCE_API_SECRET");
        let key = env!("BINANCE_API_KEY");

        // Define request params
        let timestamp = Utc::now().timestamp_millis();
        let request_path = "/sapi/v1/capital/config/getall";
        let query_string = format!("timestamp={}", timestamp);

        // Sign message
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
            .expect("Could not generate HMAC for Binance");
        mac.update(query_string.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());

        // Make url
        let url = format!(
            "{}{}?{}&signature={}",
            BINANCE_BASE_HTTP_URL, request_path, query_string, signature
        );

        // Send request
        Ok(reqwest::Client::new()
            .get(url)
            .header("X-MBX-APIKEY", key)
            .send()
            .await
            .map_err(SocketError::Http)?
            .json::<Self::NetworkInfo>()
            .await
            .map_err(SocketError::Http)?)
    }

    async fn get_usdt_pair() -> Result<Vec<(String, String)>, SocketError> {
        unimplemented!()
    }
}

/*----- */
// Stream selector
/*----- */
impl StreamSelector<BinanceSpotPublicData, OrderBookL2> for BinanceSpotPublicData {
    type Stream = BinanceSpotBookUpdate;
    type StreamTransformer =
        MultiBookTransformer<BinanceSpotPublicData, BinanceSpotBookUpdater, OrderBookL2>;
}

impl StreamSelector<BinanceSpotPublicData, Trade> for BinanceSpotPublicData {
    type Stream = BinanceTrade;
    type StreamTransformer = StatelessTransformer<BinanceSpotPublicData, Self::Stream, Trade>;
}

impl StreamSelector<BinanceSpotPublicData, AggTrades> for BinanceSpotPublicData {
    type Stream = BinanceAggTrade;
    type StreamTransformer = StatelessTransformer<BinanceSpotPublicData, Self::Stream, AggTrades>;
}
