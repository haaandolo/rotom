pub mod channel;

use std::collections::HashSet;
use channel::PoloniexChannel;
use serde_json::json;

use super::protocols::ws::WsMessage;
use super::{Connector, ExchangeStream};
use crate::error::SocketError;
use crate::exchange_connector::protocols::ws::{PingInterval, WebSocketBase, WebSocketPayload};
use crate::exchange_connector::{StreamType, Subscription};

pub struct PoloniexInterface;

impl PoloniexInterface {
    fn get_ping_interval(&self) -> PingInterval {
        PingInterval {
            time: 20,
            message: json!({"event": "ping"}),
        }
    }

    fn requests(&self, sub: &[Subscription]) -> WsMessage {
        let channels = sub
            .iter()
            .map(|s| {
                match s.stream {
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

    fn build_ws_payload(&self, sub: &[Subscription]) -> Result<WebSocketPayload, SocketError> {
        let poloniex_sub = self.requests(sub);
        let ws_payload = WebSocketPayload {
            url: PoloniexChannel::SPOT_WS_URL.as_ref().to_string(),
            subscription: Some(poloniex_sub),
            ping_interval: Some(self.get_ping_interval()),
        };

        Ok(ws_payload)
    }

    pub async fn get_stream(&self, sub: Vec<Subscription>) -> Result<ExchangeStream, SocketError> {
        let ws_payload = self.build_ws_payload(&sub)?;
        let ws_and_tasks = WebSocketBase::connect(ws_payload).await?;

        let exchange_ws = ExchangeStream {
            exchange: super::Exchange::Poloniex,
            stream: ws_and_tasks.0,
            tasks: ws_and_tasks.1,
        };

        Ok(exchange_ws)
    }
}


/*------------------------------------------- */
//Connector implenation
/*------------------------------------------- */


pub struct Poloniex;

impl Connector for Poloniex {
    fn url() -> String {
        PoloniexChannel::SPOT_WS_URL.as_ref().to_string()
    }

    fn requests(sub: &[Subscription]) -> WsMessage {
        let channels = sub
            .iter()
            .map(|s| {
                match s.stream {
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

    fn ping_interval() -> Option<PingInterval> {
        Some(PingInterval {
            time: 20,
            message: json!({"event": "ping"}),
        })
    }
}
