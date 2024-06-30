pub mod book;
pub mod channel;

use book::{PoloniexExpectedResponse, PoloniexMessage};
use channel::PoloniexChannel;
use serde_json::json;
use std::collections::HashSet;

use super::{Connector, ExchangeSub};
use crate::data::protocols::ws::{PingInterval, WsMessage};
use crate::data::{ExchangeId, StreamType};

#[derive(Debug, Default, Eq, PartialEq, Hash)]
pub struct PoloniexSpot;

impl Connector for PoloniexSpot {
    type ExchangeId = ExchangeId;
    type Message = PoloniexMessage;

    fn exchange_id(&self) -> String {
        ExchangeId::PoloniexSpot.as_str().to_string()
    }

    fn url(&self) -> String {
        PoloniexChannel::SPOT_WS_URL.as_ref().to_string()
    }

    fn requests(&self, sub: &[ExchangeSub]) -> Option<WsMessage> {
        let channels = sub
            .iter()
            .map(|s| {
                match s.stream_type {
                    StreamType::L2 => PoloniexChannel::ORDER_BOOK_L2.as_ref(),
                    StreamType::Trades => PoloniexChannel::TRADES.as_ref(),
                    _ => "Invalid Stream", // CHANGE
                }
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

    fn ping_interval(&self) -> Option<PingInterval> {
        Some(PingInterval {
            time: 30,
            message: json!({"event": "ping"}),
        })
    }

    fn expected_response(&self, subscription_response: String, sub: &[ExchangeSub]) -> bool {
        let expected_response =
            serde_json::from_str::<PoloniexExpectedResponse>(&subscription_response).unwrap();
        expected_response.symbols.len() == sub.len()
    }
}
