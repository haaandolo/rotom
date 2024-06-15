pub mod channel;
pub mod book;

use channel::PoloniexChannel;
use serde_json::json;
use std::collections::HashSet;

use super::{Connector, ExchangeSub};
use crate::data::protocols::ws::{PingInterval, WsMessage};
use crate::data::StreamType;

pub struct PoloniexSpot;

impl Connector for PoloniexSpot {
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

    fn expected_response(&self) {
        // "{\"event\":\"subscribe\",\"channel\":\"book_lv2\",\"symbols\":[\"BTC_USDT\"]}" 
    }
}
