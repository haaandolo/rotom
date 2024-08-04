pub mod channel;
pub mod l2;
pub mod market;
pub mod model;

use l2::PoloniexSpotBookUpdater;
use market::PoloniexMarket;
use serde_json::json;

use super::{Connector, StreamSelector};
use crate::data::event_models::event_book::OrderBookL2;
use crate::data::event_models::event_trade::Trades;
use crate::data::protocols::ws::{PingInterval, WsMessage};
use crate::data::shared::subscription_models::{ExchangeId, ExchangeSubscription};
use crate::data::transformer::book::MultiBookTransformer;
use crate::data::transformer::stateless_transformer::StatelessTransformer;
use channel::PoloniexChannel;
use model::{PoloniexSpotBookUpdate, PoloniexSubscriptionResponse, PoloniexTrade};

const POLONIEX_SPOT_WS_URL: &str = "wss://ws.poloniex.com/ws/public";

/*----- */
// PoloniexSpot connector
/*----- */
#[derive(Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Clone)]
pub struct PoloniexSpot;

impl Connector for PoloniexSpot {
    type ExchangeId = ExchangeId;
    type SubscriptionResponse = PoloniexSubscriptionResponse;
    type Channel = PoloniexChannel;
    type Market = PoloniexMarket;

    const ID: ExchangeId = ExchangeId::PoloniexSpot;

    fn url() -> &'static str {
        POLONIEX_SPOT_WS_URL
    }

    fn requests(
        sub: &[ExchangeSubscription<Self, Self::Channel, Self::Market>],
    ) -> Option<WsMessage> {
        let channels = sub.iter().map(|s| s.channel.as_ref()).collect::<Vec<_>>();
        let tickers = sub.iter().map(|s| s.market.as_ref()).collect::<Vec<_>>();
        let poloniex_sub = json!({
            "event": "subscribe",
            "channel": channels,
            "symbols": tickers
        });
        Some(WsMessage::text(poloniex_sub.to_string()))
    }

    fn ping_interval() -> Option<PingInterval> {
        Some(PingInterval {
            time: 30,
            message: json!({"event": "ping"}),
        })
    }
}

/*----- */
// Stream selector
/*----- */
impl StreamSelector<PoloniexSpot, OrderBookL2> for PoloniexSpot {
    type Stream = PoloniexSpotBookUpdate;
    type StreamTransformer =
        MultiBookTransformer<Self::Stream, PoloniexSpotBookUpdater, OrderBookL2>;
}

impl StreamSelector<PoloniexSpot, Trades> for PoloniexSpot {
    type Stream = PoloniexTrade;
    type StreamTransformer = StatelessTransformer<Self::Stream, Trades>;
}
