use async_trait::async_trait;
use channel::ExmoChannel;
use market::ExmoMarket;
use model::{ExmoOrderBookSnapshot, ExmoSubscriptionResponse, ExmoTrades};
use rand::Rng;
use serde_json::json;

use crate::{
    error::SocketError,
    model::{event_book_snapshot::OrderBookSnapshot, event_trade::Trades},
    protocols::ws::WsMessage,
    shared::subscription_models::{ExchangeId, ExchangeSubscription, Instrument},
    transformer::stateless_transformer::StatelessTransformer,
};

use super::{
    kucoin::model::ExmoNetworkInfo, PublicHttpConnector, PublicStreamConnector, StreamSelector,
};

pub mod channel;
pub mod market;
pub mod model;

const EXMO_SPOT_WS_URL: &str = "wss://ws-api.exmo.com:443/v1/public";

#[derive(Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Clone)]
pub struct ExmoSpotPublicData;

impl PublicStreamConnector for ExmoSpotPublicData {
    const ID: ExchangeId = ExchangeId::ExmoSpot;

    type Channel = ExmoChannel;
    type Market = ExmoMarket;
    type SubscriptionResponse = ExmoSubscriptionResponse;

    fn url() -> impl Into<String> {
        EXMO_SPOT_WS_URL
    }

    fn requests(
        subscriptions: &[ExchangeSubscription<Self, Self::Channel, Self::Market>],
    ) -> Option<WsMessage> {
        let subs = subscriptions
            .iter()
            .map(|s| format!("{}{}", s.channel.as_ref(), s.market.as_ref()))
            .collect::<Vec<_>>();

        let request = json!({
            "id": rand::thread_rng().gen::<u16>(),
            "method": "subscribe",
            "topics": subs
        });

        Some(WsMessage::text(request.to_string()))
    }

    fn expected_responses(
        subscriptions: &[ExchangeSubscription<Self, Self::Channel, Self::Market>],
    ) -> usize {
        subscriptions.len() + 1 // Plus one for successful ws connection
    }
}

/*----- */
// HttpConnector
/*----- */
pub const EXMO_BASE_HTTP_URL: &str = "https://api.exmo.com/v1.1";

#[async_trait]
impl PublicHttpConnector for ExmoSpotPublicData {
    const ID: ExchangeId = ExchangeId::ExmoSpot;

    type BookSnapShot = serde_json::Value;
    type ExchangeTickerInfo = serde_json::Value;
    type NetworkInfo = ExmoNetworkInfo;

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
        let request_path = "/payments/providers/crypto/list";
        Ok(
            reqwest::get(format!("{}{}", EXMO_BASE_HTTP_URL, request_path))
                .await
                .map_err(SocketError::Http)?
                .json::<Self::NetworkInfo>()
                .await
                .map_err(SocketError::Http)?,
        )
    }
}

/*----- */
// Stream selector
/*----- */
impl StreamSelector<ExmoSpotPublicData, OrderBookSnapshot> for ExmoSpotPublicData {
    type Stream = ExmoOrderBookSnapshot;
    type StreamTransformer =
        StatelessTransformer<ExmoSpotPublicData, Self::Stream, OrderBookSnapshot>;
}

impl StreamSelector<ExmoSpotPublicData, Trades> for ExmoSpotPublicData {
    type Stream = ExmoTrades;
    type StreamTransformer = StatelessTransformer<ExmoSpotPublicData, Self::Stream, Trades>;
}
