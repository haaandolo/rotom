
use exchange_connector::Connector;

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