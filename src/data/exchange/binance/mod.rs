pub mod channel;
pub mod l2;
pub mod market;
pub mod model;

use channel::BinanceChannel;
use l2::BinanceSpotBookUpdater;
use market::BinanceMarket;
use model::{BinanceSpotBookUpdate, BinanceSubscriptionResponse, BinanceTrade};
use serde_json::json;

use super::{Connector, StreamSelector};
use crate::data::{
    event_models::{event_book::OrderBookL2, event_trade::Trades}, protocols::ws::WsMessage, shared::subscription_models::{ExchangeId, ExchangeSubscription}, transformer::{book::MultiBookTransformer, stateless_transformer::StatelessTransformer}
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
            .map(|s| format!("{}{}", s.market.as_ref(), s.channel.as_ref()).to_lowercase())
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

    // fn validate_subscription(subscription_response: String, _number_of_tickers: usize) -> bool {
    //     let subscription_response =
    //         serde_json::from_str::<BinanceSubscriptionResponse>(&subscription_response).unwrap();
    //     subscription_response.result.is_none()
    // }
}

/*----- */
// Stream selector
/*----- */
impl StreamSelector<BinanceSpot, OrderBookL2> for BinanceSpot {
    type Stream = BinanceSpotBookUpdate;
    type StreamTransformer =
        MultiBookTransformer<Self::Stream, BinanceSpotBookUpdater, OrderBookL2>;
}

impl StreamSelector<BinanceSpot, Trades> for BinanceSpot {
    type Stream = BinanceTrade;
    type StreamTransformer = StatelessTransformer<Self::Stream, Trades>;
}
