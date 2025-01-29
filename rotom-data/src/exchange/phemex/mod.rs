use async_trait::async_trait;
use channel::PhemexChannel;
use l2::PhemexSpotBookUpdater;
use market::PhemexMarket;
use model::{
    PhemexOrderBookUpdate, PhemexSubscriptionResponse, PhemexTickerInfo, PhemexTradesUpdate,
};
use rand::Rng;
use serde_json::json;

use crate::{
    error::SocketError,
    model::{event_book::OrderBookL2, event_trade::Trades},
    protocols::ws::WsMessage,
    shared::subscription_models::{ExchangeId, ExchangeSubscription, Instrument},
    transformer::{book::MultiBookTransformer, stateless_transformer::StatelessTransformer},
};

use super::{PublicHttpConnector, PublicStreamConnector, StreamSelector};

pub mod channel;
pub mod l2;
pub mod market;
pub mod model;

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

        println!("request: {}", request);

        Some(WsMessage::Text(request.to_string()))
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
    type NetworkInfo = serde_json::Value;

    async fn get_book_snapshot(_instrument: Instrument) -> Result<Self::BookSnapShot, SocketError> {
        unimplemented!()
    }

    async fn get_ticker_info(
        _instrument: Instrument,
    ) -> Result<Self::ExchangeTickerInfo, SocketError> {
        let request_path = "/public/products";
        let ticker_info_url = format!("{}{}", PHEMEX_BASE_HTTP_URL, request_path,);

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
