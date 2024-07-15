pub mod book;
pub mod channel;

use book::{PoloniexBook, PoloniexSubscriptionResponse, PoloniexTrade};
use channel::PoloniexChannel;
use serde_json::json;
use std::collections::HashSet;

use super::{Connector, Instrument, StreamSelector};
use crate::data::models::trade::Trades;
use crate::data::protocols::ws::ws_client::{PingInterval, WsMessage};
use crate::data::models::book::OrderBookL2;
use crate::data::models::subs::{ExchangeId, StreamType};

#[derive(Debug, Default, Eq, PartialEq, Hash)]
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
                _ => panic!("Invalid stream was enter for poloniex"),
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

impl StreamSelector<PoloniexSpot, OrderBookL2> for PoloniexSpot {
    type Stream = PoloniexBook;
}

impl StreamSelector<PoloniexSpot, Trades> for PoloniexSpot {
    type Stream = PoloniexTrade;
}