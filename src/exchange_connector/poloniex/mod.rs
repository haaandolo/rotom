use std::collections::HashSet;

use serde_json::json;

use crate::error::Result;
use crate::exchange_connector::ws::{PingInterval, WebSocketBase, WebSocketPayload};
use crate::exchange_connector::{StreamType, Subscription};

use super::ExchangeStream;

// CHANNELS
#[derive(Debug)]
pub struct PoloniexChannel(pub &'static str);

impl PoloniexChannel {
    pub const SPOT_WS_URL: Self = Self("wss://ws.poloniex.com/ws/public");
    pub const TRADES: Self = Self("trades");
    pub const ORDER_BOOK_L2: Self = Self("book_lv2");
}

impl AsRef<str> for PoloniexChannel {
    fn as_ref(&self) -> &str {
        self.0
    }
}

pub struct PoloniexInterface;

impl PoloniexInterface {
    pub async fn get_stream(&self, sub: Vec<Subscription>) -> Result<ExchangeStream> {
        let channels = sub
            .iter()
            .map(|s| {
                let stream = match s.stream {
                    StreamType::L2 => PoloniexChannel::ORDER_BOOK_L2.as_ref(),
                    StreamType::Trades => PoloniexChannel::TRADES.as_ref(),
                    _ => "Invalid Stream", // CHANGE
                };
                stream
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

        let ws_payload = WebSocketPayload {
            url: PoloniexChannel::SPOT_WS_URL.as_ref(),
            subscription: Some(poloniex_sub),
            ping_interval: Some(self.get_ping_interval()),
        };

        let ws = WebSocketBase::connect(ws_payload).await?;

        let exchange_ws = ExchangeStream {
            exchange: super::Exchange::Poloniex,
            stream: ws,
        };

        Ok(exchange_ws)
    }

    pub fn get_ping_interval(&self) -> PingInterval {
        PingInterval {
            time: 20,
            message: json!({"event": "ping"}),
        }
    }
}
