pub mod channel;
pub mod l2;
pub mod market;
pub mod model;

use async_trait::async_trait;
use channel::PhemexChannel;
use l2::PhemexSpotBookUpdater;
use market::PhemexMarket;
use model::{
    PhemexNetworkInfo, PhemexNetworkInfoData, PhemexOrderBookUpdate, PhemexSubscriptionResponse,
    PhemexTickerInfo, PhemexTradesUpdate,
};
use rand::Rng;
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;

use crate::{
    error::SocketError,
    model::{event_book::OrderBookL2, event_trade::Trades},
    protocols::ws::{PingInterval, WsMessage},
    shared::subscription_models::{ExchangeId, ExchangeSubscription, Instrument},
    transformer::{book::MultiBookTransformer, stateless_transformer::StatelessTransformer},
};

use super::{PublicHttpConnector, PublicStreamConnector, StreamSelector};

#[derive(Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Clone)]
pub struct PhemexSpotPublicData;

/*----- */
// Stream connector
/*----- */
const PHEMEX_SPOT_WS_URL: &str = "wss://ws.phemex.com";

impl PublicStreamConnector for PhemexSpotPublicData {
    const ID: ExchangeId = ExchangeId::PhemexSpot;

    type SubscriptionResponse = PhemexSubscriptionResponse;
    type Channel = PhemexChannel;
    type Market = PhemexMarket;

    fn url() -> impl Into<String> {
        PHEMEX_SPOT_WS_URL
    }

    // Note: Phemex can only have one ticker per connection
    fn requests(
        subscriptions: &[ExchangeSubscription<Self, Self::Channel, Self::Market>],
    ) -> Option<WsMessage> {
        let channel = &subscriptions[0].channel; // One channel type per Vec<ExchangeSubscription>

        let subs = subscriptions
            .iter()
            .map(|s| s.market.as_ref())
            .collect::<Vec<&str>>();

        let request = json!({
            "id": rand::thread_rng().gen::<u64>(),
            "method": channel.0,
            "params": subs,
        });

        Some(WsMessage::Text(request.to_string()))
    }

    fn ping_interval() -> Option<PingInterval> {
        Some(PingInterval {
            time: 25,
            message: json!({"event": "ping"}),
        })
    }
}

/*----- */
// HttpConnector
/*----- */
pub const PHEMEX_BASE_HTTP_URL: &str = "https://api.phemex.com";

#[async_trait]
impl PublicHttpConnector for PhemexSpotPublicData {
    const ID: ExchangeId = ExchangeId::PhemexSpot;

    type BookSnapShot = serde_json::Value;
    type ExchangeTickerInfo = PhemexTickerInfo;
    type NetworkInfo = Vec<PhemexNetworkInfoData>;

    async fn get_book_snapshot(_instrument: Instrument) -> Result<Self::BookSnapShot, SocketError> {
        unimplemented!()
    }

    async fn get_ticker_info(
        _instrument: Instrument,
    ) -> Result<Self::ExchangeTickerInfo, SocketError> {
        let request_path = "/public/products";
        let ticker_info_url = format!("{}{}", PHEMEX_BASE_HTTP_URL, request_path);

        reqwest::get(ticker_info_url)
            .await
            .map_err(SocketError::Http)?
            .json::<Self::ExchangeTickerInfo>()
            .await
            .map_err(SocketError::Http)
    }

    async fn get_network_info(
        instruments: Vec<Instrument>,
    ) -> Result<Self::NetworkInfo, SocketError> {
        let mut network_info = Vec::with_capacity(instruments.len());

        let instruments_chunked = instruments
            .into_iter()
            .collect::<Vec<_>>()
            .chunks(5)
            .map(|c| c.to_vec())
            .collect::<Vec<Vec<_>>>();

        for chunk in instruments_chunked.into_iter() {
            let network_info_futures = chunk
                .into_iter()
                .map(|instrument| {
                    let request_path = format!(
                        "/exchange/public/cfg/chain-settings?currency={}",
                        instrument.base.to_uppercase()
                    );
                    let ticker_info_url = format!("{}{}", PHEMEX_BASE_HTTP_URL, request_path);
                    get_single_network_info::<PhemexNetworkInfo>(ticker_info_url)
                })
                .collect::<Vec<_>>();

            let network_info_result = futures::future::join_all(network_info_futures)
                .await
                .into_iter()
                .filter_map(|result| result.ok())
                .collect::<Vec<_>>();

            network_info.push(network_info_result);
            tokio::time::sleep(Duration::from_millis(1000)).await;
        }

        Ok(network_info
            .into_iter()
            .flatten()
            .filter_map(|network_info| network_info.data)
            .collect::<Vec<Vec<PhemexNetworkInfoData>>>()
            .into_iter()
            .flatten()
            .collect::<Vec<PhemexNetworkInfoData>>())
    }
}

// Can't do async closures so this func is required
async fn get_single_network_info<DeStruct: for<'de> Deserialize<'de>>(
    url: String,
) -> Result<DeStruct, SocketError> {
    reqwest::get(url)
        .await
        .map_err(SocketError::Http)?
        .json::<DeStruct>()
        .await
        .map_err(SocketError::Http)
}

/*----- */
// Stream selector
/*----- */
impl StreamSelector<PhemexSpotPublicData, OrderBookL2> for PhemexSpotPublicData {
    type Stream = PhemexOrderBookUpdate;
    type StreamTransformer =
        MultiBookTransformer<PhemexSpotPublicData, PhemexSpotBookUpdater, OrderBookL2>;
}

impl StreamSelector<PhemexSpotPublicData, Trades> for PhemexSpotPublicData {
    type Stream = PhemexTradesUpdate;
    type StreamTransformer = StatelessTransformer<PhemexSpotPublicData, Self::Stream, Trades>;
}

// let coins = Self::get_ticker_info(Instrument::default())
//     .await?
//     .data
//     .currencies
//     .into_iter()
//     .filter(|currency| currency.status == "Listed")
//     .map(|currency_filtered| currency_filtered.currency)
//     .collect::<Vec<String>>();
