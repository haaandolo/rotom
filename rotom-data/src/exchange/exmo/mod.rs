use channel::ExmoChannel;
use market::ExmoMarket;
use model::{ExmoOrderBookSnapshot, ExmoSubscriptionResponse, ExmoTrades};
use rand::Rng;
use serde_json::json;

use crate::{
    model::{event_book_snapshot::OrderBookSnapshot, event_trade::Trades},
    protocols::ws::{ WsMessage},
    shared::subscription_models::{ExchangeId, ExchangeSubscription},
    transformer::stateless_transformer::StatelessTransformer,
};

use super::{PublicStreamConnector, StreamSelector};

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
