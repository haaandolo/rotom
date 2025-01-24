use channel::OkxChannel;
use market::OkxMarket;
use model::{OkxOrderBookSnapshot, OkxSubscriptionResponse, OkxTrade};
use serde_json::json;

use crate::{
    model::{event_book_snapshot::OrderBookSnapshot, event_trade::Trades},
    protocols::ws::WsMessage,
    shared::subscription_models::{ExchangeId, ExchangeSubscription},
    transformer::stateless_transformer::StatelessTransformer,
};

use super::{PublicStreamConnector, StreamSelector};

pub mod channel;
pub mod market;
pub mod model;

#[derive(Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Clone)]
pub struct OkxSpotPublicData;

const OKX_SPOT_WS_URL: &str = "wss://wspap.okx.com:8443/ws/v5/public";

impl PublicStreamConnector for OkxSpotPublicData {
    const ID: ExchangeId = ExchangeId::OkxSpot;

    type Channel = OkxChannel;
    type Market = OkxMarket;
    type SubscriptionResponse = OkxSubscriptionResponse;

    fn url() -> &'static str {
        OKX_SPOT_WS_URL
    }

    fn requests(
        subscriptions: &[ExchangeSubscription<Self, Self::Channel, Self::Market>],
    ) -> Option<WsMessage> {
        let subs = subscriptions
            .iter()
            .map(|s| json!({"channel": s.channel.as_ref(), "instId": s.market.as_ref()}))
            .collect::<Vec<_>>();

        let request = json!({
            "op": "subscribe",
            "args": subs
        });

        Some(WsMessage::text(request.to_string()))
    }
}

/*----- */
// Stream selector
/*----- */
impl StreamSelector<OkxSpotPublicData, OrderBookSnapshot> for OkxSpotPublicData {
    type Stream = OkxOrderBookSnapshot;
    type StreamTransformer =
        StatelessTransformer<OkxSpotPublicData, Self::Stream, OrderBookSnapshot>;
}

impl StreamSelector<OkxSpotPublicData, Trades> for OkxSpotPublicData {
    type Stream = OkxTrade;
    type StreamTransformer = StatelessTransformer<OkxSpotPublicData, Self::Stream, Trades>;
}
