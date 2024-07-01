pub mod book;
pub mod channel;

use book::{PoloniexMessage, PoloniexSubscriptionResponse};
use channel::PoloniexChannel;
use serde_json::json;
use std::collections::HashSet;

use super::{Connector, Instrument};
use crate::data::protocols::ws::{PingInterval, WsMessage};
use crate::data::shared::orderbook::Event;
use crate::data::{ExchangeId, StreamType};

#[derive(Debug, Default, Eq, PartialEq, Hash)]
pub struct PoloniexSpot;

impl Connector for PoloniexSpot {
    type ExchangeId = ExchangeId;
    type Input = PoloniexMessage;
    type Output = Event;
    type SubscriptionResponse = PoloniexSubscriptionResponse;

    fn exchange_id(&self) -> String {
        ExchangeId::PoloniexSpot.as_str().to_string()
    }

    fn url(&self) -> String {
        PoloniexChannel::SPOT_WS_URL.as_ref().to_string()
    }

    fn requests(&self, sub: &[Instrument]) -> Option<WsMessage> {
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

    fn validate_subscription(&self, subscription_response: String, sub: &[Instrument]) -> bool {
        let subscription_reponse=
            serde_json::from_str::<PoloniexSubscriptionResponse>(&subscription_response).unwrap();
        subscription_reponse.symbols.len() == sub.len()
    }

    fn transform(&mut self, input: Self::Input) -> Self::Output {
        match input {
            PoloniexMessage::Book(book) => Event::from(book),
            PoloniexMessage::Trade(trade) => Event::from(trade),
            // PoloniexMessage::ExpectedResponse(_) => {
            //     Event::new("ok".to_string(), 0, 0, None, None, None, None)
            // }
        }
    }
}
