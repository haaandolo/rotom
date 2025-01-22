use channel::WooxChannel;
use market::WooxMarket;
use model::{WooxOrderBookSnapshot, WooxSubscriptionResponse};
use rand::Rng;
use serde_json::json;

use crate::{
    model::event_book_snapshot::OrderBookSnapshot,
    protocols::ws::{PingInterval, WsMessage},
    shared::subscription_models::{ExchangeId, ExchangeSubscription},
    transformer::stateless_transformer::StatelessTransformer,
};

use super::{PublicStreamConnector, StreamSelector};

pub mod channel;
pub mod market;
pub mod model;

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
            time: 9,
            message: json!({ "pong": rand::thread_rng().gen::<u64>() }),
        })
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
