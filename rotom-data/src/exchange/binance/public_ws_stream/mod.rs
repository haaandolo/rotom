pub mod channel;
pub mod l2;
pub mod market;
pub mod model;

use channel::BinanceChannel;
use l2::BinanceSpotBookUpdater;
use market::BinanceMarket;
use model::{BinanceAggTrade, BinanceSpotBookUpdate, BinanceSubscriptionResponse, BinanceTrade};
use serde_json::json;

use crate::{
    model::{
        event_book::OrderBookL2,
        event_trade::{AggTrades, Trades},
    },
    exchange::{Connector, StreamSelector},
    protocols::ws::WsMessage,
    shared::subscription_models::{ExchangeId, ExchangeSubscription},
    transformer::{book::MultiBookTransformer, stateless_transformer::StatelessTransformer},
};

const BINANCE_SPOT_WS_URL: &str = "wss://stream.binance.com:9443/ws";

/*----- */
// BinanceSpot connector
/*----- */
#[derive(Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Clone)]
pub struct BinanceSpot;

impl Connector for BinanceSpot {
    type ExchangeId = ExchangeId;
    type SubscriptionResponse = BinanceSubscriptionResponse;
    type Channel = BinanceChannel;
    type Market = BinanceMarket;

    const ID: ExchangeId = ExchangeId::BinanceSpot;

    fn url() -> &'static str {
        BINANCE_SPOT_WS_URL
    }

    fn requests(
        subscriptions: &[ExchangeSubscription<Self, Self::Channel, Self::Market>],
    ) -> Option<WsMessage> {
        let binance_subs = subscriptions
            .iter()
            .map(|s| format!("{}{}", s.market.as_ref().to_lowercase(), s.channel.as_ref()))
            .collect::<Vec<_>>();

        let binance_request = json!({
            "method": "SUBSCRIBE",
            "params": binance_subs,
            "id": 1
        });

        Some(WsMessage::Text(binance_request.to_string()))
    }

    fn expected_responses(
        _subscriptions: &[ExchangeSubscription<Self, Self::Channel, Self::Market>],
    ) -> usize
    where
        Self: Sized,
    {
        1
    }
}

/*----- */
// Stream selector
/*----- */
impl StreamSelector<BinanceSpot, OrderBookL2> for BinanceSpot {
    type Stream = BinanceSpotBookUpdate;
    type StreamTransformer = MultiBookTransformer<BinanceSpot, BinanceSpotBookUpdater, OrderBookL2>;
}

impl StreamSelector<BinanceSpot, Trades> for BinanceSpot {
    type Stream = BinanceTrade;
    type StreamTransformer = StatelessTransformer<BinanceSpot, Self::Stream, Trades>;
}

impl StreamSelector<BinanceSpot, AggTrades> for BinanceSpot {
    type Stream = BinanceAggTrade;
    type StreamTransformer = StatelessTransformer<BinanceSpot, Self::Stream, AggTrades>;
}
