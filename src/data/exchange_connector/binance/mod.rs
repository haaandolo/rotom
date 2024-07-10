pub mod book;
pub mod channel;

use book::{BinanceBook, BinanceSubscriptionResponse};
use channel::BinanceChannel;
use serde_json::json;
use std::collections::HashSet;

use super::{Connector, Identifier, Instrument, StreamSelector};
use crate::data::{
    protocols::ws::ws_client::WsMessage, ExchangeId, ExchangeSub, OrderbookL2, StreamType, Subscription,
};

#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct BinanceSpot;

impl Connector for BinanceSpot {
    type ExchangeId = ExchangeId;
    type SubscriptionResponse = BinanceSubscriptionResponse;

    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::BinanceSpot
    }

    fn url(&self) -> String {
        BinanceChannel::SPOT_WS_URL.as_ref().to_string()
    }

    fn requests(&self, subscriptions: &[Instrument]) -> Option<WsMessage> {
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

    fn validate_subscription(
        &self,
        subscription_response: String,
        _subscriptions: &[Instrument],
    ) -> bool {
        let subscription_response =
            serde_json::from_str::<BinanceSubscriptionResponse>(&subscription_response).unwrap();
        subscription_response.result.is_none()
    }
}

impl Identifier<String> for Subscription<BinanceSpot, OrderbookL2> {
    fn id(&self) -> String {
        String::from("worked !")
    }
}

impl StreamSelector<BinanceSpot, OrderbookL2> for ExchangeSub<BinanceSpot, OrderbookL2> {
    type DeStruct = BinanceBook;
}
