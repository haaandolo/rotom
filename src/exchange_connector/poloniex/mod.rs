use std::collections::HashSet;

use serde_json::json;

use crate::error::{NetworkErrors, Result};
use crate::exchange_connector::ws::WebSocketBase;
use crate::exchange_connector::{StreamType, Subscription};

// CHANNELS
#[derive(Debug)]
pub struct PoloniexChannel(pub &'static str);

impl PoloniexChannel {
    pub const TRADES: Self = Self("trades");
    pub const ORDER_BOOK_L2: Self = Self("book_lv2");
}

impl AsRef<str> for PoloniexChannel {
    fn as_ref(&self) -> &str {
        self.0
    }
}

pub trait Identifier<T> {
    fn id(&self) -> T;
}

impl Identifier<PoloniexChannel> for Subscription {
    fn id(&self) -> PoloniexChannel {
        PoloniexChannel::TRADES
    }
}

// INTERFACE STRUCT
pub struct PoloniexExchangeInterface;

impl PoloniexExchangeInterface {
    pub async fn get_stream(sub: Vec<Subscription>) -> Result<()> {
        let ws_url = "wss://ws.poloniex.com/ws/public";

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

        println!("{:#?}", channels);
        println!("{:#?}", tickers);

        let poloniex_sub = json!({
            "event": "subscribe",
            "channel": channels,
            "symbols": tickers
        });

        Ok(())
    }
}
