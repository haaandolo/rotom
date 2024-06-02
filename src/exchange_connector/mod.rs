pub mod binance;
pub mod poloniex;
pub mod protocols;
pub mod subscribe;

// use std::fmt;

use protocols::ws::{FuturesTokio, PingInterval, WsMessage, WsRead};

#[derive(Debug, PartialEq, Hash, Eq, Clone, Copy)]
pub enum Exchange {
    BinanceSpot,
    Poloniex,
}

#[derive(Debug)]
pub enum StreamType {
    L1,
    L2,
    Trades,
}

#[derive(Debug)]
pub enum MarketType {
    Spot,
    Futures,
}

#[derive(Debug)]
pub struct Subscription {
    pub exchange: Exchange,
    pub base: String,
    pub quote: String,
    pub market: MarketType,
    pub stream: StreamType,
}

impl Subscription {
    pub fn new(
        _exchange: Exchange,
        _base: String,
        _quote: String,
        _market_type: MarketType,
        _stream: StreamType,
    ) -> Self {
        Self {
            exchange: _exchange,
            base: _base,
            quote: _quote,
            market: _market_type,
            stream: _stream,
        }
    }
}

/*---------- */
#[derive(Debug)]
pub struct ExchangeSub {
    pub base: &'static str,
    pub quote: &'static str,
    pub stream_type: StreamType,
}

#[derive(Debug)]
pub struct SubGeneric {
    pub exchange: Exchange,
    pub base: &'static str,
    pub quote: &'static str,
    pub stream_type: StreamType,
}

impl SubGeneric {
    pub fn new(_exchange: Exchange, _base: &'static str, _quote: &'static str, _stream_type: StreamType) -> Self {
        Self {
            exchange: _exchange,
            base: _base,
            quote: _quote,
            stream_type: _stream_type,
        }
    }

    pub fn convert_subscription(self) -> ExchangeSub {
        ExchangeSub {
            base: self.base,
            quote: self.quote,
            stream_type: self.stream_type
        }
    }
}

// impl<E: std::fmt::Display, S: std::fmt::Display> fmt::Debug for SubGeneric<E, S> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(
//             f,
//             "Subscription: {}, {}, {}",
//             self.exchange, self.base, self.quote
//         )
//     }
// }

/*---------- */
#[derive(Debug)]
pub struct ExchangeStream {
    pub exchange: Exchange,
    pub stream: WsRead,
    pub tasks: Vec<FuturesTokio>,
}

pub trait Identifier<T> {
    fn id(&self) -> T;
}

pub trait Connector {
    fn url(&self) -> String;

    fn ping_interval() -> Option<PingInterval> {
        None
    }

    fn requests(subscriptions: &[ExchangeSub]) -> WsMessage;

    fn expected_response() -> Option<usize> {
        None
    }
}
