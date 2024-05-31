pub mod binance;
pub mod poloniex;
pub mod ws;

use ws::WsRead;

#[derive(Debug)]
pub enum Exchange {
    Binance,
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

pub struct ExchangeStream {
    pub exchange: Exchange,
    pub stream: WsRead,
}

pub trait Identifier<T> {
    fn id(&self) -> T;
}