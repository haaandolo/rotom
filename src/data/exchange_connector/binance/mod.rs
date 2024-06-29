pub mod book;
pub mod channel;

use book::{BinanceExpectedResponse, BinanceMessage};
use channel::BinanceChannel;
use serde_json::json;
use std::collections::HashSet;

use super::{Connector, ExchangeSub};
use crate::data::{protocols::ws::WsMessage, ExchangeId, StreamType};

#[derive(Debug, Default, Eq, PartialEq, Hash)]
pub struct BinanceSpot;

impl Connector for BinanceSpot {
    type ExchangeId = ExchangeId;
    type Message = BinanceMessage;

    fn exchange_id(&self) -> String {
        ExchangeId::BinanceSpot.as_str().to_string()
    }

    fn url(&self) -> String {
        BinanceChannel::SPOT_WS_URL.as_ref().to_string()
    }

    fn requests(&self, subscriptions: &[ExchangeSub]) -> Option<WsMessage> {
        let channels = subscriptions
            .iter()
            .map(|s| {
                let stream = match s.stream_type {
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

        Some(WsMessage::Text(binance_request.to_string()))
    }

    fn expected_response(
        &self,
        subscription_response: String,
        subscriptions: &[ExchangeSub],
    ) -> bool {
        let response_struct =
            serde_json::from_str::<BinanceExpectedResponse>(&subscription_response).unwrap();
        response_struct.result.is_none()
    }
}
