pub mod channel;
pub mod l2;
pub mod market;
pub mod model;

use l2::PoloniexSpotBookUpdater;
use serde_json::json;
use std::collections::HashSet;

use super::{Connector, Instrument, StreamSelector};
use crate::data::model::event_book::OrderBookL2;
use crate::data::model::subs::{ExchangeId, StreamType};
use crate::data::model::event_trade::Trades;
use crate::data::protocols::ws::{PingInterval, WsMessage};
use crate::data::transformer::book::MultiBookTransformer;
use crate::data::transformer::stateless_transformer::StatelessTransformer;
use channel::PoloniexChannel;
use model::{PoloniexSpotBookUpdate, PoloniexSubscriptionResponse, PoloniexTrade};

/*----- */
// Poloniex connector
/*----- */
#[derive(Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct PoloniexSpot;

impl Connector for PoloniexSpot {
    type ExchangeId = ExchangeId;
    type SubscriptionResponse = PoloniexSubscriptionResponse;

    const ID: ExchangeId = ExchangeId::PoloniexSpot;

    fn url() -> String {
        PoloniexChannel::SPOT_WS_URL.as_ref().to_string()
    }

    fn requests(sub: &[Instrument]) -> Option<WsMessage> {
        let channels = sub
            .iter()
            .map(|s| match s.stream_type {
                StreamType::L2 => PoloniexChannel::ORDER_BOOK_L2.as_ref(),
                StreamType::Trades => PoloniexChannel::TRADES.as_ref(),
                _ => panic!("Invalid stream was enter for poloniex"), // TODO
            })
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        let tickers = sub
            .iter()
            .map(|s| format!("{}_{}", s.base, s.quote))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

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

    fn validate_subscription(subscription_response: String, sub: &[Instrument]) -> bool {
        let subscription_response =
            serde_json::from_str::<PoloniexSubscriptionResponse>(&subscription_response).unwrap();
        subscription_response.symbols.len() == sub.len()
    }
}

/*----- */
// Stream selector
/*----- */
// impl StreamSelector<PoloniexSpot, OrderBookL2> for PoloniexSpot {
//     type Stream = PoloniexSpotBookUpdate;
//     type StreamTransformer = StatelessTransformer<Self::Stream, OrderBookL2>;
// }

impl StreamSelector<PoloniexSpot, OrderBookL2> for PoloniexSpot{
    type Stream = PoloniexSpotBookUpdate;
    type StreamTransformer =
        MultiBookTransformer<Self::Stream, PoloniexSpotBookUpdater, OrderBookL2>;
}

impl StreamSelector<PoloniexSpot, Trades> for PoloniexSpot {
    type Stream = PoloniexTrade;
    type StreamTransformer = StatelessTransformer<Self::Stream, Trades>;
}
