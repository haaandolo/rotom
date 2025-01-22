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
    model::{event_book_snapshot::OrderBookSnapshot, event_trade::Trades},
    protocols::ws::{PingInterval, WsMessage},
    shared::subscription_models::{ExchangeId, ExchangeSubscription, Instrument},
    transformer::stateless_transformer::StatelessTransformer,
};

use super::{PublicHttpConnector, PublicStreamConnector, StreamSelector};


#[derive(Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Clone)]
pub struct WooxSpotPublicData;

const WOOX_SPOT_WS_URL: &str = "wss://wss.woox.io/ws/stream/8a6152d8-3f34-42fa-9a23-0aae9fa34208";

impl PublicStreamConnector for WooxSpotPublicData {
    const ID: ExchangeId = ExchangeId::WooxSpot;

    type Channel = WooxChannel;
    type Market = WooxMarket;
    type SubscriptionResponse = WooxSubscriptionResponse;

    fn url() -> &'static str {
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
}

/*----- */
// WooxSpot HttpConnector
/*----- */
pub const HTTP_NETWORK_INFO_URL_WOOX_SPOT: &str = "https://api.woox.io/v1/public/token_network";

#[async_trait]
impl PublicHttpConnector for WooxSpotPublicData {
    const ID: ExchangeId = ExchangeId::HtxSpot;

    type BookSnapShot = serde_json::Value;
    type ExchangeTickerInfo = serde_json::Value;
    type NetworkInfo = WooxNetworkInfo; // todo

    async fn get_book_snapshot(_instrument: Instrument) -> Result<Self::BookSnapShot, SocketError> {
        unimplemented!()
    }

    async fn get_ticker_info(
        _instrument: Instrument,
    ) -> Result<Self::ExchangeTickerInfo, SocketError> {
        unimplemented!()
    }

    async fn get_network_info() -> Result<Self::NetworkInfo, SocketError> {
        Ok(reqwest::get(HTTP_NETWORK_INFO_URL_WOOX_SPOT)
            .await
            .map_err(SocketError::Http)?
            .json::<Self::NetworkInfo>()
            .await
            .map_err(SocketError::Http)?)
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

impl StreamSelector<WooxSpotPublicData, Trades> for WooxSpotPublicData {
    type Stream = WooxTrade;
    type StreamTransformer = StatelessTransformer<WooxSpotPublicData, Self::Stream, Trades>;
}
