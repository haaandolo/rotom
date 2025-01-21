pub mod channel;
pub mod market;
pub mod model;

use channel::HtxChannel;
use market::HtxMarket;
use model::{HtxOrderBookSnapshot, HtxSubscriptionResponse};
use rand::Rng;
use serde_json::json;

use crate::{
    model::event_book_snapshot::OrderBookSnapshot,
    protocols::ws::{PingInterval, WsMessage},
    shared::subscription_models::{ExchangeId, ExchangeSubscription},
    transformer::stateless_transformer::StatelessTransformer,
};

use super::{PublicStreamConnector, StreamSelector};

const HTX_SPOT_WS_URL: &str = "wss://api.huobi.pro/feed";

#[derive(Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Clone)]
pub struct HtxSpotPublicData;

impl PublicStreamConnector for HtxSpotPublicData {
    const ID: ExchangeId = ExchangeId::HtxSpot;

    type Channel = HtxChannel;
    type Market = HtxMarket;
    type SubscriptionResponse = HtxSubscriptionResponse;

    fn url() -> &'static str {
        HTX_SPOT_WS_URL
    }

    fn requests(
        subscriptions: &[ExchangeSubscription<Self, Self::Channel, Self::Market>],
    ) -> Option<WsMessage> {
        let htx_subs = subscriptions
            .iter()
            .map(|s| format!("market.{}.{}", s.market.as_ref(), s.channel.as_ref()))
            .collect::<Vec<_>>();

        let htx_request = json!({
            "sub": htx_subs,
            "id": rand::thread_rng().gen::<u64>().to_string(),
        });

        Some(WsMessage::text(htx_request.to_string()))
    }

    fn ping_interval() -> Option<PingInterval> {
        Some(PingInterval {
            time: 4,
            message: json!({ "pong": rand::thread_rng().gen::<u64>() }),
        })
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
