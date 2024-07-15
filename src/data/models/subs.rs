
/*----- */
// Exchange subscription
/*----- */
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Instrument {
    pub base: String,
    pub quote: String,
    pub stream_type: StreamType,
}

impl Instrument {
    pub fn new(base: String, quote: String, stream_type: StreamType) -> Self {
        Self {
            base,
            quote,
            stream_type,
        }
    }
}

impl From<(String, String, StreamType)> for Instrument {
    fn from((base, quote, stream_type): (String, String, StreamType)) -> Self {
        Self::new(base, quote, stream_type)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Subscription<Exchange, Kind> {
    pub exchange: Exchange,
    pub instrument: Instrument,
    pub kind: Kind,
}

impl<Exchange, S, Kind> From<(Exchange, S, S, StreamType, Kind)> for Subscription<Exchange, Kind>
where
    S: Into<String>,
{
    fn from(
        (exchange, base, quote, stream_type, kind): (Exchange, S, S, StreamType, Kind),
    ) -> Self {
        Self {
            exchange,
            instrument: Instrument::new(base.into(), quote.into(), stream_type),
            kind,
        }
    }
}

/*----- */
// Exchange ID's & stream types
/*----- */
#[derive(Debug, PartialEq, Hash, Eq, Clone, Copy, Ord, PartialOrd)]
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

#[derive(Debug, PartialEq, Clone, Hash, Eq, Ord, PartialOrd)]
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