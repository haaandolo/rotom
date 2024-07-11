pub mod book;
pub mod channel;

use book::{BinanceBook, BinanceSubscriptionResponse};
use channel::BinanceChannel;
use serde_json::json;
use std::collections::HashSet;

use super::{Connector, Instrument, StreamSelector};
use crate::data::{protocols::ws::ws_client::WsMessage, ExchangeId, OrderbookL2, StreamType};

#[derive(Debug, Default, Eq, PartialEq, Hash)]
pub struct BinanceSpot;

impl Connector for BinanceSpot {
    type ExchangeId = ExchangeId;
    type SubscriptionResponse = BinanceSubscriptionResponse;

    const ID: ExchangeId = ExchangeId::BinanceSpot;

    fn url() -> String {
        BinanceChannel::SPOT_WS_URL.as_ref().to_string()
    }

    fn requests(subscriptions: &[Instrument]) -> Option<WsMessage> {
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

    fn validate_subscription(subscription_response: String, _subscriptions: &[Instrument]) -> bool {
        let subscription_response =
            serde_json::from_str::<BinanceSubscriptionResponse>(&subscription_response).unwrap();
        subscription_response.result.is_none()
    }
}

impl StreamSelector<BinanceSpot, OrderbookL2> for BinanceSpot {
    type Stream = BinanceBook;
}
