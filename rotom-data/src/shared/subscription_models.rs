use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::exchange::{Identifier, PublicStreamConnector};

/*----- */
// Instrument model
/*----- */
#[derive(Default, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct Instrument {
    pub base: String,
    pub quote: String,
}

impl Instrument {
    pub fn new<S>(base: S, quote: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            base: base.into(),
            quote: quote.into(),
        }
    }
}

impl Display for Instrument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.base, self.quote)
    }
}

impl From<(String, String)> for Instrument {
    fn from((base, quote): (String, String)) -> Self {
        Self::new(base, quote)
    }
}

/*----- */
// Subscription model
/*----- */
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Subscription<Exchange, StreamKind> {
    pub exchange: Exchange,
    pub instrument: Instrument,
    pub stream_kind: StreamKind,
}

impl<Exchange, S, StreamKind> From<(Exchange, S, S, StreamKind)>
    for Subscription<Exchange, StreamKind>
where
    S: Into<String>,
{
    fn from((exchange, base, quote, stream_kind): (Exchange, S, S, StreamKind)) -> Self {
        Self {
            exchange,
            instrument: Instrument::new(base.into(), quote.into()),
            stream_kind,
        }
    }
}

impl<Exchange, StreamKind> Subscription<Exchange, StreamKind> {
    pub fn new<I>(exchange: Exchange, instrument: I, stream_kind: StreamKind) -> Self
    where
        I: Into<Instrument>,
    {
        Self {
            exchange,
            instrument: instrument.into(),
            stream_kind,
        }
    }
}

/*----- */
// Exchange subscription model
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
    Exchange: PublicStreamConnector + Clone,
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
// Exchange IDs
/*----- */
#[derive(Debug, PartialEq, Hash, Eq, Clone, Copy, Ord, PartialOrd, Deserialize, Serialize)]
pub enum ExchangeId {
    BinanceSpot,
    PoloniexSpot,
    HtxSpot,
    WooxSpot,
    BitstampSpot,
    CoinExSpot,
    OkxSpot,
    KuCoinSpot
}

impl ExchangeId {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExchangeId::BinanceSpot => "binancespot",
            ExchangeId::PoloniexSpot => "poloniexspot",
            ExchangeId::HtxSpot => "htxspot",
            ExchangeId::WooxSpot => "wooxspot",
            ExchangeId::BitstampSpot => "bitstampspot",
            ExchangeId::CoinExSpot => "coinexspot",
            ExchangeId::OkxSpot => "okxspot",
            ExchangeId::KuCoinSpot=> "kucoinspot",
        }
    }
}

impl Display for ExchangeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

//*----- */
// Stream kind
//*----- */
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Default, Copy)]
pub enum StreamKind {
    Trade,
    Trades,
    #[default]
    L2,
    AggTrades,
    Snapshot,
}

impl StreamKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            StreamKind::Trade => "trade",
            StreamKind::Trades => "trades",
            StreamKind::L2 => "l2",
            StreamKind::AggTrades => "agg_trade",
            StreamKind::Snapshot => "snapshot",
        }
    }
}

impl Display for StreamKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
