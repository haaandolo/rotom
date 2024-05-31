use std::collections::HashSet;

use crate::{error::Result, exchange_connector::ws::WebSocketPayload};

use super::{ws::WebSocketBase, ExchangeStream, StreamType, Subscription};

// CHANNELS
pub struct BinanceChannel(pub &'static str);

impl BinanceChannel {
    pub const WS_URL: Self = Self("wss://stream.binance.com:9443/stream?streams=");
    pub const TRADES: Self = Self("@trade");
    pub const ORDER_BOOK_L1: Self = Self("@bookTicker");
    pub const ORDER_BOOK_L2: Self = Self("@depth@100ms");
    pub const LIQUIDATIONS: Self = Self("@forceOrder");
}

impl AsRef<str> for BinanceChannel {
    fn as_ref(&self) -> &str {
        self.0
    }
}

pub struct BinanceInterface;

impl BinanceInterface {
    pub async fn get_stream(&self, sub: Vec<Subscription>) -> Result<ExchangeStream> {
        let channels = sub
            .iter()
            .map(|s| {
                let stream = match s.stream {
                    StreamType::L1 => BinanceChannel::ORDER_BOOK_L1.as_ref(),
                    StreamType::L2 => BinanceChannel::ORDER_BOOK_L2.as_ref(),
                    StreamType::Trades => BinanceChannel::TRADES.as_ref(),
                };
                let ticker = format!("{}{}{}", s.base, s.quote, stream).to_lowercase();
                ticker
            })
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        let _url = format!("{}{}", BinanceChannel::WS_URL.as_ref(), channels.join("/"));

        let ws_payload = WebSocketPayload {
            url: &_url,
            subscription: None,
            ping_interval: None,
        };

        let ws = WebSocketBase::connect(ws_payload).await?;

        let exchange_ws = ExchangeStream {
            exchange: super::Exchange::Binance,
            stream: ws,
        };

        Ok(exchange_ws)
    }
}
