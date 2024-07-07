use std::marker::PhantomData;

use exchange_connector::Connector;
use serde::{Deserialize, Serialize};

pub mod exchange_connector;
pub mod protocols;
pub mod shared;
pub mod subscriber;

/*----- */
// Exchange ID's & stream types
/*----- */
#[derive(Debug, PartialEq, Hash, Eq, Clone, Copy)]
pub enum ExchangeId {
    BinanceSpot,
    PoloniexSpot,
}

impl ExchangeId {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExchangeId::BinanceSpot => "binancespot",
            ExchangeId::PoloniexSpot => "poloniexspot",
        }
    }
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum StreamType {
    L1,
    L2,
    Trades,
}

impl StreamType {
    pub fn as_str(&self) -> &'static str {
        match self {
            StreamType::L1 => "l1",
            StreamType::L2 => "l2",
            StreamType::Trades => "trade",
        }
    }
}

/*----- */
// Exchange subscription
/*----- */
#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct Instrument {
    pub base: &'static str,
    pub quote: &'static str,
    pub stream_type: StreamType,
}

impl Instrument {
    pub fn new(_base: &'static str, _quote: &'static str, _stream_type: StreamType) -> Self {
        Self {
            base: _base,
            quote: _quote,
            stream_type: _stream_type,
        }
    }
}

#[derive(Clone)]
pub struct Subscription<ExchangeConnector> {
    pub connector: ExchangeConnector,
    pub instruments: Vec<Instrument>,
}

impl<ExchangeConnector> Subscription<ExchangeConnector>
where
    ExchangeConnector: Connector,
{
    pub fn new(_connector: ExchangeConnector, _instruments: Vec<Instrument>) -> Self {
        Self {
            connector: _connector,
            instruments: _instruments,
        }
    }
}

/*-------------- */
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]

pub struct Instrument2<Kind> {
    pub base: String,
    pub quote: String,
    pub stream_type: PhantomData<Kind>,
}

impl<Kind> Instrument2<Kind> {
    pub fn new(_base: String, _quote: String, _stream_type: Kind) -> Self {
        Self {
            base: _base,
            quote: _quote,
            stream_type: PhantomData,
        }
    }
}

impl<Kind> From<(String, String, Kind)> for Instrument2<Kind> {
    fn from((base, quote, kind): (String, String, Kind)) -> Self {
        Self::new(base, quote, kind)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct Subscription2<Exchange, Kind> {
    pub exchange: Exchange,
    #[serde(flatten)]
    pub instrument: Instrument2<Kind>,
}

impl<Exchange, S, Kind> From<(Exchange, S, S, Kind)> for Subscription2<Exchange, Kind>
where
    S: Into<String>,
{
    fn from((exchange, base, quote, kind): (Exchange, S, S, Kind)) -> Self {
        Self {
            exchange,
            instrument: Instrument2::new(base.into(), quote.into(), kind),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let test = (ExchangeId::BinanceSpot, "arb", "usdt", StreamType::L2);
        let some = Subscription2::from(test);
        println!("{:#?}", some);
    }
}
