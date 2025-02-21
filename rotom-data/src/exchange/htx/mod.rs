pub mod channel;
pub mod market;
pub mod model;

use async_trait::async_trait;
use channel::HtxChannel;
use market::HtxMarket;
use model::{HtxNetworkInfo, HtxOrderBookSnapshot, HtxSubscriptionResponse, HtxTrade};
use rand::Rng;
use serde_json::json;

use crate::{
    error::SocketError,
    model::{event_book_snapshot::OrderBookSnapshot, event_trade::Trades},
    protocols::ws::{PingInterval, WsMessage},
    shared::subscription_models::{ExchangeId, ExchangeSubscription, Instrument},
    transformer::stateless_transformer::StatelessTransformer,
};

use super::{PublicHttpConnector, PublicStreamConnector, StreamSelector};

const HTX_SPOT_WS_URL: &str = "wss://api-aws.huobi.pro/ws";

#[derive(Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Clone)]
pub struct HtxSpotPublicData;

impl PublicStreamConnector for HtxSpotPublicData {
    const ID: ExchangeId = ExchangeId::HtxSpot;

    type Channel = HtxChannel;
    type Market = HtxMarket;
    type SubscriptionResponse = HtxSubscriptionResponse;

    fn url() -> impl Into<String> {
        HTX_SPOT_WS_URL
    }

    fn requests(
        subscriptions: &[ExchangeSubscription<Self, Self::Channel, Self::Market>],
    ) -> Option<WsMessage> {
        let subs = subscriptions
            .iter()
            .map(|s| format!("market.{}.{}", s.market.as_ref(), s.channel.as_ref()))
            .collect::<Vec<_>>();

        let request = json!({
            "sub": subs,
            "id": rand::thread_rng().gen::<u64>().to_string(),
        });

        Some(WsMessage::text(request.to_string()))
    }

    fn ping_interval() -> Option<PingInterval> {
        Some(PingInterval {
            time: 4,
            message: json!({ "pong": rand::thread_rng().gen::<u64>() }),
        })
    }
}

/*----- */
// HtxSpot HttpConnector
/*----- */
pub const HTX_BASE_HTTP_URL: &str = "https://api-aws.huobi.pro";

#[async_trait]
impl PublicHttpConnector for HtxSpotPublicData {
    const ID: ExchangeId = ExchangeId::HtxSpot;

    type BookSnapShot = serde_json::Value;
    type ExchangeTickerInfo = serde_json::Value;
    type NetworkInfo = HtxNetworkInfo;

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
        let request_path = "/v2/reference/currencies";
        Ok(
            reqwest::get(format!("{}{}", HTX_BASE_HTTP_URL, request_path))
                .await
                .map_err(SocketError::Http)?
                .json::<Self::NetworkInfo>()
                .await
                .map_err(SocketError::Http)?,
        )
    }

    async fn get_usdt_pair() -> Result<Vec<(String, String)>, SocketError> {
        let request_path = "/v1/settings/common/market-symbols";

        let response = reqwest::get(format!("{}{}", HTX_BASE_HTTP_URL, request_path))
            .await
            .map_err(SocketError::Http)?
            .json::<serde_json::Value>()
            .await
            .map_err(SocketError::Http)?;

        let tickers = response["data"]
            .as_array()
            .unwrap() // unwrap allowed
            .iter()
            .filter_map(|ticker| {
                if ticker["qc"].as_str().unwrap().to_lowercase() == "usdt"
                    && ticker["state"] == "online"
                {
                    Some((
                        ticker["bc"].as_str().unwrap().to_lowercase(), // unwrap allowed
                        ticker["qc"].as_str().unwrap().to_lowercase(), // unwrap allowed
                    ))
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
impl StreamSelector<HtxSpotPublicData, OrderBookSnapshot> for HtxSpotPublicData {
    type Stream = HtxOrderBookSnapshot;
    type StreamTransformer =
        StatelessTransformer<HtxSpotPublicData, Self::Stream, OrderBookSnapshot>;
}

impl StreamSelector<HtxSpotPublicData, Trades> for HtxSpotPublicData {
    type Stream = HtxTrade;
    type StreamTransformer = StatelessTransformer<HtxSpotPublicData, Self::Stream, Trades>;
}
