pub mod ws;
pub mod poloniex;


pub enum Exchange {
    Binance,
    Poloniex
}

pub enum StreamType {
    L1,
    L2,
    Trades   
}

pub enum MarketType {
    Spot,
    Futures
}

pub enum Subscription2<S> {
    New(Exchange, S, S, MarketType, StreamType)
}

pub struct Subscription {
    pub exchange: Exchange,
    pub base: String,
    pub quote: String,
    pub market: MarketType,
    pub stream: StreamType 
}