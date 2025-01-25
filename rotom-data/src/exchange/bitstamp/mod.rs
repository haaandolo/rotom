use channel::BitstampChannel;
use market::BitstampMarket;
use model::{BitstampOrderBookSnapshot, BitstampSubscriptionResponse, BitstampTrade};
use serde_json::json;

use crate::{
    model::{event_book_snapshot::OrderBookSnapshot, event_trade::Trade},
    protocols::ws::WsMessage,
    shared::subscription_models::{ExchangeId, ExchangeSubscription},
    transformer::stateless_transformer::StatelessTransformer,
};

use super::{PublicStreamConnector, StreamSelector};

pub mod channel;
pub mod market;
pub mod model;

#[derive(Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Clone)]
pub struct BitstampSpotPublicData;

const BITSTAMP_SPOT_WS_URL: &str = "wss://ws.bitstamp.net";

impl PublicStreamConnector for BitstampSpotPublicData {
    const ID: ExchangeId = ExchangeId::BitstampSpot;

    type Channel = BitstampChannel;
    type Market = BitstampMarket;
    type SubscriptionResponse = BitstampSubscriptionResponse;

    fn url() -> impl Into<String> {
        BITSTAMP_SPOT_WS_URL
    }

    // Bitstamp can only have one socket per ticker so when initiating, have one ticker per vector
    fn requests(
        subscriptions: &[ExchangeSubscription<Self, Self::Channel, Self::Market>],
    ) -> Option<WsMessage> {
        let subs = subscriptions
            .iter()
            .map(|s| format!("{}{}", s.channel.as_ref(), s.market.as_ref()))
            .collect::<Vec<_>>();

        let request = json!({
            "event": "bts:subscribe",
            "data": {
                "channel": subs[0], // only allowed one ticker per ws connection so index works
            }
        });

        println!("requests: {:?}", request);

        Some(WsMessage::text(request.to_string()))
    }
}

/*----- */
// Stream selector
/*----- */
impl StreamSelector<BitstampSpotPublicData, OrderBookSnapshot> for BitstampSpotPublicData {
    type Stream = BitstampOrderBookSnapshot;
    type StreamTransformer =
        StatelessTransformer<BitstampSpotPublicData, Self::Stream, OrderBookSnapshot>;
}

impl StreamSelector<BitstampSpotPublicData, Trade> for BitstampSpotPublicData {
    type Stream = BitstampTrade;
    type StreamTransformer = StatelessTransformer<BitstampSpotPublicData, Self::Stream, Trade>;
}
