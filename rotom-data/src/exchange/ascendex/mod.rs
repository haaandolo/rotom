pub mod channel;
pub mod l2;
pub mod market;
pub mod model;

use async_trait::async_trait;
use channel::AscendExChannel;
use l2::AscendExSpotBookUpdater;
use market::AscendExMarket;
use model::{
    AscendExBookUpdate, AscendExNetworkInfo, AscendExOrderBookSnapshot,
    AscendExSubscriptionResponse, AscendExTickerInfo, AscendExTrades,
};
use serde_json::json;

use crate::{
    error::SocketError,
    model::{event_book::OrderBookL2, event_trade::Trades},
    protocols::ws::{PingInterval, WsMessage},
    shared::subscription_models::{ExchangeId, ExchangeSubscription, Instrument, StreamKind},
    transformer::{book::MultiBookTransformer, stateless_transformer::StatelessTransformer},
};

use super::{PublicHttpConnector, PublicStreamConnector, StreamSelector};

const ASCENDEX_SPOT_WS_URL: &str = "wss://ascendex.com/7/api/pro/v1/stream";

/*----- */
// Stream Connector
/*----- */
#[derive(Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Clone)]
pub struct AscendExSpotPublicData;

impl PublicStreamConnector for AscendExSpotPublicData {
    const ID: ExchangeId = ExchangeId::AscendExSpot;
    const ORDERBOOK: StreamKind = StreamKind::L2;
    const TRADE: StreamKind = StreamKind::Trades;

    type Channel = AscendExChannel;
    type Market = AscendExMarket;
    type SubscriptionResponse = AscendExSubscriptionResponse;

    fn url() -> impl Into<String> {
        ASCENDEX_SPOT_WS_URL
    }

    fn requests(
        subscriptions: &[ExchangeSubscription<Self, Self::Channel, Self::Market>],
    ) -> Option<WsMessage> {
        let channel = &subscriptions[0].channel;

        let subs = subscriptions
            .iter()
            .map(|s| s.market.as_ref())
            .collect::<Vec<&str>>()
            .join(",");

        let request_param = format!("{}{}", channel.0, subs);

        let request = json!({
            "op": "sub",
            "id": uuid::Uuid::new_v4(),
            "ch": request_param
        });

        Some(WsMessage::text(request.to_string()))
    }

    fn ping_interval() -> Option<PingInterval> {
        Some(PingInterval {
            time: 15,
            message: json!({"op": "ping"}),
        })
    }
}

/*----- */
// HttpConnector
/*----- */
pub const ASCENDEX_BASE_HTTP_URL: &str = "https://ascendex.com";

#[async_trait]
impl PublicHttpConnector for AscendExSpotPublicData {
    const ID: ExchangeId = ExchangeId::AscendExSpot;

    type BookSnapShot = AscendExOrderBookSnapshot;
    type ExchangeTickerInfo = AscendExTickerInfo;
    type NetworkInfo = AscendExNetworkInfo;

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

    async fn get_network_info(
        _instruments: Vec<Instrument>,
    ) -> Result<Self::NetworkInfo, SocketError> {
        let request_path = "/api/pro/v2/assets";
        let network_info_url = format!("{}{}", ASCENDEX_BASE_HTTP_URL, request_path);
        reqwest::get(network_info_url)
            .await
            .map_err(SocketError::Http)?
            .json::<Self::NetworkInfo>()
            .await
            .map_err(SocketError::Http)
    }

    async fn get_usdt_pair() -> Result<Vec<(String, String)>, SocketError> {
        let request_path = "/api/pro/v1/spot/ticker";

        let response = reqwest::get(format!("{}{}", ASCENDEX_BASE_HTTP_URL, request_path))
            .await
            .map_err(SocketError::Http)?
            .json::<serde_json::Value>()
            .await
            .map_err(SocketError::Http)?;

        println!("{}", response);

        let volume_threshold = Self::get_volume_threshold();
        let tickers: Vec<(String, String)> = response["data"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|ticker| {
                let ticker_lower = ticker["symbol"].as_str().unwrap().to_lowercase();
                let volume = ticker["volume"]
                    .as_str()
                    .unwrap_or("0")
                    .trim_matches('"') // This removes quotation marks
                    .parse::<u64>()
                    .unwrap_or(0);

                let mut ticker_split = ticker_lower.split("/");
                let base = ticker_split.next().unwrap_or("").to_string();
                let quote = ticker_split.next().unwrap_or("").to_string();
                if quote == "usdt" && volume > volume_threshold {
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
impl StreamSelector<AscendExSpotPublicData, OrderBookL2> for AscendExSpotPublicData {
    type Stream = AscendExBookUpdate;
    type StreamTransformer =
        MultiBookTransformer<AscendExSpotPublicData, AscendExSpotBookUpdater, OrderBookL2>;
}

impl StreamSelector<AscendExSpotPublicData, Trades> for AscendExSpotPublicData {
    type Stream = AscendExTrades;
    type StreamTransformer = StatelessTransformer<AscendExSpotPublicData, Self::Stream, Trades>;
}
