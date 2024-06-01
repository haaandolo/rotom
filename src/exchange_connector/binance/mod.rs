pub mod channel;

use channel::BinanceChannel;
use serde_json::json;
use std::collections::HashSet;

use super::{
    protocols::ws::{WebSocketBase, WsMessage},
    Connector, ExchangeStream, StreamType, Subscription,
};
use crate::{error::SocketError, exchange_connector::protocols::ws::WebSocketPayload};


pub struct BinanceInterface;

impl BinanceInterface {
    fn requests(&self, sub: &[Subscription]) -> WsMessage {
        let channels = sub
            .iter()
            .map(|s| {
                let stream = match s.stream {
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

        WsMessage::Text(binance_request.to_string())
    }

    fn build_ws_payload(&self, sub: &[Subscription]) -> Result<WebSocketPayload, SocketError> {
        let sub = self.requests(sub);

        let ws_payload = WebSocketPayload {
            url: BinanceChannel::SPOT_WS_URL.as_ref().to_string(),
            subscription: Some(sub),
            ping_interval: None,
        };

        Ok(ws_payload)
    }

    pub async fn get_stream(&self, sub: Vec<Subscription>) -> Result<ExchangeStream, SocketError> {
        let ws_payload = self.build_ws_payload(&sub)?;
        let ws_and_tasks = WebSocketBase::connect(ws_payload).await?;

        let exchange_ws = ExchangeStream {
            exchange: super::Exchange::Binance,
            stream: ws_and_tasks.0,
            tasks: ws_and_tasks.1,
        };

        Ok(exchange_ws)
    }
}

/*------------------------------------------- */
//Connector implenation
/*------------------------------------------- */

pub struct Binance;

impl Connector for Binance {
    fn url() -> String {
        BinanceChannel::SPOT_WS_URL.as_ref().to_string()
    }

    fn requests(subscriptions: &[Subscription]) -> WsMessage {
        let channels = subscriptions
            .iter()
            .map(|s| {
                let stream = match s.stream {
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

        WsMessage::Text(binance_request.to_string())
    }
}
