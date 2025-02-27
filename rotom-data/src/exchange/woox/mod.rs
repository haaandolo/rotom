pub mod channel;
pub mod market;
pub mod model;

use async_trait::async_trait;
use channel::WooxChannel;
use market::WooxMarket;
use model::{WooxNetworkInfo, WooxOrderBookSnapshot, WooxSubscriptionResponse, WooxTrade};
use rand::Rng;
use serde_json::json;

use crate::{
    error::SocketError,
    model::{event_book_snapshot::OrderBookSnapshot, event_trade::Trade},
    protocols::ws::{PingInterval, WsMessage},
    shared::subscription_models::{ExchangeId, ExchangeSubscription, Instrument, StreamKind},
    transformer::stateless_transformer::StatelessTransformer,
};

use super::{PublicHttpConnector, PublicStreamConnector, StreamSelector};

#[derive(Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Clone)]
pub struct WooxSpotPublicData;

const WOOX_SPOT_WS_URL: &str = "wss://wss.woox.io/ws/stream/8a6152d8-3f34-42fa-9a23-0aae9fa34208";

impl PublicStreamConnector for WooxSpotPublicData {
    const ID: ExchangeId = ExchangeId::WooxSpot;
    const ORDERBOOK: StreamKind = StreamKind::Snapshot;
    const TRADE: StreamKind = StreamKind::Trade;

    type Channel = WooxChannel;
    type Market = WooxMarket;
    type SubscriptionResponse = WooxSubscriptionResponse;

    fn url() -> impl Into<String> {
        WOOX_SPOT_WS_URL
    }

    // Woox can only have one socket per ticker so when initiating, have one ticker per vector
    fn requests(
        subscriptions: &[ExchangeSubscription<Self, Self::Channel, Self::Market>],
    ) -> Option<WsMessage> {
        let subs = subscriptions
            .iter()
            .map(|s| format!("{}{}", s.market.as_ref(), s.channel.as_ref()))
            .collect::<Vec<_>>();

        let request = json!({
            "id": rand::thread_rng().gen::<u64>().to_string(),
            "topic": subs[0], // Note: woo can only connect to one stream per socket so this indexing works
            "event": "subscribe"
        });

        Some(WsMessage::text(request.to_string()))
    }

    fn ping_interval() -> Option<PingInterval> {
        Some(PingInterval {
            time: 10,
            message: json!({ "event": "ping" }),
        })
    }

    fn ws_chunk_size() -> usize {
        1
    }
}

/*----- */
// WooxSpot HttpConnector
/*----- */
pub const WOOX_BASE_HTTP_URL: &str = "https://api.woox.io";

#[async_trait]
impl PublicHttpConnector for WooxSpotPublicData {
    const ID: ExchangeId = ExchangeId::WooxSpot;

    type BookSnapShot = serde_json::Value;
    type ExchangeTickerInfo = serde_json::Value;
    type NetworkInfo = WooxNetworkInfo;

    async fn get_book_snapshot(_instrument: Instrument) -> Result<Self::BookSnapShot, SocketError> {
        unimplemented!()
    }

    async fn get_ticker_info(
        _instrument: Instrument,
    ) -> Result<Self::ExchangeTickerInfo, SocketError> {
        unimplemented!()
    }

    async fn get_network_info(
        _instruments: Vec<Instrument>,
    ) -> Result<Self::NetworkInfo, SocketError> {
        let request_path = "/v1/public/token_network";
        Ok(
            reqwest::get(format!("{}{}", WOOX_BASE_HTTP_URL, request_path))
                .await
                .map_err(SocketError::Http)?
                .json::<Self::NetworkInfo>()
                .await
                .map_err(SocketError::Http)?,
        )
    }

    async fn get_usdt_pair() -> Result<Vec<(String, String)>, SocketError> {
        let request_path = "/v1/public/info";

        let response = reqwest::get(format!("{}{}", WOOX_BASE_HTTP_URL, request_path))
            .await
            .map_err(SocketError::Http)?
            .json::<serde_json::Value>()
            .await
            .map_err(SocketError::Http)?;

        let tickers = response["rows"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|ticker| {
                let status = ticker["status"].as_str().unwrap().to_string();
                let symbol = ticker["symbol"].as_str().unwrap().to_lowercase();

                let mut ticker_split = symbol.split("_"); // comes like SPOT_BTC_USDT or PERP_BTC_USDT
                let ticker_kind = ticker_split.next().unwrap_or("").to_string();
                let base = ticker_split.next().unwrap_or("").to_string();
                let quote = ticker_split.next().unwrap_or("").to_string();

                if ticker_kind == "spot" && status == "TRADING" && quote == "usdt" {
                    Some((base, quote))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        Ok(tickers)
    }
}

/*----- */
// Stream selector
/*----- */
impl StreamSelector<WooxSpotPublicData, OrderBookSnapshot> for WooxSpotPublicData {
    type Stream = WooxOrderBookSnapshot;
    type StreamTransformer =
        StatelessTransformer<WooxSpotPublicData, Self::Stream, OrderBookSnapshot>;
}

impl StreamSelector<WooxSpotPublicData, Trade> for WooxSpotPublicData {
    type Stream = WooxTrade;
    type StreamTransformer = StatelessTransformer<WooxSpotPublicData, Self::Stream, Trade>;
}
