pub mod channel;
pub mod market;
pub mod model;

use async_trait::async_trait;
use channel::KuCoinChannel;
use market::KuCoinMarket;
use model::{
    KuCoinNetworkInfo, KuCoinOrderBookSnapshot, KuCoinSubscriptionResponse, KuCoinTrade,
    KuCoinWsUrl,
};
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
pub struct KuCoinSpotPublicData;

impl PublicStreamConnector for KuCoinSpotPublicData {
    const ID: ExchangeId = ExchangeId::KuCoinSpot;
    const ORDERBOOK: StreamKind = StreamKind::Snapshot;
    const TRADE: StreamKind = StreamKind::Trade;

    type Channel = KuCoinChannel;
    type Market = KuCoinMarket;
    type SubscriptionResponse = KuCoinSubscriptionResponse;

    fn url() -> impl Into<String> {
        let base_url_http = "https://api.kucoin.com";
        let token_post = "/api/v1/bullet-public";

        let kucoin_ws_url_response = tokio::task::block_in_place(|| {
            reqwest::blocking::Client::new()
                .post(format!("{}{}", base_url_http, token_post))
                .send()
                .map_err(SocketError::Http)
                .unwrap()
                .json::<KuCoinWsUrl>()
                .map_err(SocketError::Http)
                .unwrap()
        });

        format!(
            "{}?token={}&[connectId={}]",
            kucoin_ws_url_response.data.instance_servers[0].endpoint,
            kucoin_ws_url_response.data.token,
            uuid::Uuid::new_v4()
        )
    }

    fn requests(
        subscriptions: &[ExchangeSubscription<Self, Self::Channel, Self::Market>],
    ) -> Option<WsMessage> {
        let channel = subscriptions[0].channel; // Each vec of subs can be of one channel type so this works
        let subs = subscriptions
            .iter()
            .map(|s| s.market.as_ref())
            .collect::<Vec<_>>()
            .join(",");

        let sub_request = format!("{}{}", channel.0, subs);
        let request = json!({
            "id": uuid::Uuid::new_v4(),
            "type": "subscribe",
            "topic": sub_request,
            "response": true
        });

        Some(WsMessage::text(request.to_string()))
    }

    fn expected_responses(
        _subscriptions: &[ExchangeSubscription<Self, Self::Channel, Self::Market>],
    ) -> usize {
        // 2 because first response is a message stating the ws connected succesfully.
        // The second is the success message for subbing to channels
        2
    }

    fn ping_interval() -> Option<PingInterval> {
        Some(PingInterval {
            time: 30,
            message: json!({ "id":  uuid::Uuid::new_v4(), "type": "ping"}),
        })
    }
}

/*----- */
// KuCoin HttpConnector
/*----- */
pub const KUCOIN_BASE_HTTP_URL: &str = "https://api.kucoin.com";

#[async_trait]
impl PublicHttpConnector for KuCoinSpotPublicData {
    const ID: ExchangeId = ExchangeId::KuCoinSpot;

    type BookSnapShot = serde_json::Value;
    type ExchangeTickerInfo = serde_json::Value;
    type NetworkInfo = KuCoinNetworkInfo;

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
        let request_path = "/api/v3/currencies";
        Ok(
            reqwest::get(format!("{}{}", KUCOIN_BASE_HTTP_URL, request_path))
                .await
                .map_err(SocketError::Http)?
                .json::<Self::NetworkInfo>()
                .await
                .map_err(SocketError::Http)?,
        )
    }

    async fn get_usdt_pair() -> Result<Vec<(String, String)>, SocketError> {
        let request_path = "/api/v2/symbols";

        let response = reqwest::get(format!("{}{}", KUCOIN_BASE_HTTP_URL, request_path))
            .await
            .map_err(SocketError::Http)?
            .json::<serde_json::Value>()
            .await
            .map_err(SocketError::Http)?;

        let tickers = response["data"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|ticker| {
                let base = ticker["baseCurrency"].as_str().unwrap().to_lowercase();
                let quote = ticker["quoteCurrency"].as_str().unwrap().to_lowercase();
                let status = ticker["enableTrading"].as_bool().unwrap();
                if quote == "usdt" && status {
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
impl StreamSelector<KuCoinSpotPublicData, OrderBookSnapshot> for KuCoinSpotPublicData {
    type Stream = KuCoinOrderBookSnapshot;
    type StreamTransformer =
        StatelessTransformer<KuCoinSpotPublicData, Self::Stream, OrderBookSnapshot>;
}

impl StreamSelector<KuCoinSpotPublicData, Trade> for KuCoinSpotPublicData {
    type Stream = KuCoinTrade;
    type StreamTransformer = StatelessTransformer<KuCoinSpotPublicData, Self::Stream, Trade>;
}
