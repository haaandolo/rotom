pub mod channel;
pub mod l2;
pub mod market;
pub mod model;

use channel::BinanceChannel;
use l2::BinanceSpotBookUpdater;
use model::{BinanceSpotBookUpdate, BinanceSubscriptionResponse, BinanceTrade};
use serde_json::json;
use std::collections::HashSet;

use super::{Connector, Instrument, StreamSelector};
use crate::data::{
    model::{
        event_book::OrderBookL2,
        subs::{ExchangeId, StreamType},
        event_trade::Trades,
    },
    protocols::ws::WsMessage,
    transformer::{book::MultiBookTransformer, stateless_transformer::StatelessTransformer},
};

/*----- */
// Binance connector
/*----- */
#[derive(Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct BinanceSpot;

impl Connector for BinanceSpot {
    type ExchangeId = ExchangeId;
    type SubscriptionResponse = BinanceSubscriptionResponse;

    const ID: ExchangeId = ExchangeId::BinanceSpot;

    fn url() -> String {
        BinanceChannel::SPOT_WS_URL.as_ref().to_string()
    }

    fn requests(subscriptions: &[Instrument]) -> Option<WsMessage> {
        let channels = subscriptions
            .iter()
            .map(|s| {
                let stream = match s.stream_type {
                    StreamType::L1 => BinanceChannel::ORDER_BOOK_L1.as_ref(),
                    StreamType::L2 => BinanceChannel::ORDER_BOOK_L2.as_ref(),
                    StreamType::Trades => BinanceChannel::TRADES.as_ref(),
                };
                format!("{}{}{}", s.base, s.quote, stream).to_lowercase()
            })
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        let binance_request = json!({
            "method": "SUBSCRIBE",
            "params": channels,
            "id": 1
        });

        Some(WsMessage::Text(binance_request.to_string()))
    }

    fn validate_subscription(subscription_response: String, _subscriptions: &[Instrument]) -> bool {
        let subscription_response =
            serde_json::from_str::<BinanceSubscriptionResponse>(&subscription_response).unwrap();
        subscription_response.result.is_none()
    }
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
