pub mod channel;

use channel::PoloniexChannel;
use serde_json::json;
use std::collections::HashSet;

use super::protocols::ws::WsMessage;
use super::{Connector, ExchangeSub};
use crate::exchange_connector::protocols::ws::PingInterval;
use crate::exchange_connector::StreamType;

pub struct PoloniexSpot;

impl Connector for PoloniexSpot {
    fn url(&self) -> String {
        PoloniexChannel::SPOT_WS_URL.as_ref().to_string()
    }

    fn requests(&self, sub: &[ExchangeSub]) -> WsMessage {
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

        WsMessage::text(poloniex_sub.to_string())
    }

    fn ping_interval(&self) -> Option<PingInterval> {
        Some(PingInterval {
            time: 20,
            message: json!({"event": "ping"}),
        })
    }
}
