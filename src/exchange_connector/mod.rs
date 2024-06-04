pub mod binance;
pub mod poloniex;
pub mod protocols;
pub mod subscribe;

use protocols::ws::{FuturesTokio, PingInterval, WsMessage};

// Subscription enum inputs
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

// Exchange subscription
#[derive(Debug)]
pub struct ExchangeSub {
    pub base: &'static str,
    pub quote: &'static str,
    pub stream_type: StreamType,
}

#[derive(Debug)]
pub struct Sub {
    pub exchange: Exchange,
    pub base: &'static str,
    pub quote: &'static str,
    pub stream_type: StreamType,
}

impl Sub {
    pub fn new(
        _exchange: Exchange,
        _base: &'static str,
        _quote: &'static str,
        _stream_type: StreamType,
    ) -> Self {
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
            stream_type: self.stream_type,
        }
    }
}

// Exchange connector traits
pub trait Identifier<T> {
    fn id(&self) -> T;
}

pub trait Connector {
    fn url(&self) -> String;

    fn ping_interval(&self) -> Option<PingInterval> {
        None
    }

    fn requests(&self, subscriptions: &[ExchangeSub]) -> WsMessage;

    fn expected_response(&self) -> Option<usize> {
        None
    }
}
