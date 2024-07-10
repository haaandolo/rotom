pub mod book;
pub mod channel;

use book::{PoloniexBook, PoloniexSubscriptionResponse};
use channel::PoloniexChannel;
use serde_json::json;
use std::collections::HashSet;

use super::{Connector, Instrument, StreamSelector};
use crate::data::protocols::ws::ws_client::{PingInterval, WsMessage};
use crate::data::{ExchangeId, ExchangeSub, OrderbookL2, StreamType};

#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct PoloniexSpot;

impl Connector for PoloniexSpot {
    type ExchangeId = ExchangeId;
    type SubscriptionResponse = PoloniexSubscriptionResponse;

    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::PoloniexSpot
    }

    fn url(&self) -> String {
        PoloniexChannel::SPOT_WS_URL.as_ref().to_string()
    }

    fn requests(&self, sub: &[Instrument]) -> Option<WsMessage> {
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

    fn ping_interval(&self) -> Option<PingInterval> {
        Some(PingInterval {
            time: 30,
            message: json!({"event": "ping"}),
        })
    }

    // Note: Poloniex sends a subscription validation response for
    // each stream type. This will cause the websocket to panic as
    // the validation code for websocket only looks at the first
    // message from the WS. Hence, only group by singular stream.
    fn validate_subscription(&self, subscription_response: String, sub: &[Instrument]) -> bool {
        let subscription_response =
            serde_json::from_str::<PoloniexSubscriptionResponse>(&subscription_response).unwrap();
        subscription_response.symbols.len() == sub.len()
    }
}

impl StreamSelector<PoloniexSpot, OrderbookL2> for ExchangeSub<PoloniexSpot, OrderbookL2> {
    type DeStruct = PoloniexBook;
}
