use std::fmt::Display;

use crate::data::exchange::{Connector, Identifier};

/*----- */
// Exchange subscription
/*----- */
#[derive(Default, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
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

impl Display for Instrument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}, {}", self.base, self.quote, self.stream_type)
    }
}

impl From<(String, String, StreamType)> for Instrument {
    fn from((base, quote, stream_type): (String, String, StreamType)) -> Self {
        Self::new(base, quote, stream_type)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Subscription<Exchange, StreamKind> {
    pub exchange: Exchange,
    pub instrument: Instrument,
    pub kind: StreamKind,
}

impl<Exchange, S, StreamKind> From<(Exchange, S, S, StreamType, StreamKind)>
    for Subscription<Exchange, StreamKind>
where
    S: Into<String>,
{
    fn from(
        (exchange, base, quote, stream_type, kind): (Exchange, S, S, StreamType, StreamKind),
    ) -> Self {
        Self {
            exchange,
            instrument: Instrument::new(base.into(), quote.into(), stream_type),
            kind,
        }
    }
}

/*----- */
// Internal exchange subscription
/*----- */
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ExchangeSubscription<Exchange, Channel, Market> {
    pub exchange: Exchange,
    pub channel: Channel,
    pub market: Market,
    pub instrument: Instrument,
}

impl<Exchange> ExchangeSubscription<Exchange, Exchange::Channel, Exchange::Market>
where
    Exchange: Connector + Clone,
{
    pub fn new<StreamKind>(subs: &Subscription<Exchange, StreamKind>) -> Self
    where
        Subscription<Exchange, StreamKind>:
            Identifier<Exchange::Channel> + Identifier<Exchange::Market>,
    {
        Self {
            exchange: subs.exchange.clone(),
            channel: subs.id(),
            market: subs.id(),
            instrument: subs.instrument.clone(),
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
    Default,
}

impl ExchangeId {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExchangeId::BinanceSpot => "binancespot",
            ExchangeId::PoloniexSpot => "poloniexspot",
            ExchangeId::Default => "default",
        }
    }
}

impl Display for ExchangeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Default, Debug, PartialEq, Clone, Hash, Eq, Ord, PartialOrd)]
pub enum StreamType {
    L1,
    L2,
    Trades,
    #[default]
    Default,
}

impl StreamType {
    pub fn as_str(&self) -> &'static str {
        match self {
            StreamType::L1 => "l1",
            StreamType::L2 => "l2",
            StreamType::Trades => "trade",
            StreamType::Default => "default",
        }
    }
}

impl Display for StreamType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
