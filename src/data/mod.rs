use std::fmt;

pub mod exchange_connector;
pub mod protocols;
pub mod subscriber;
pub mod shared;
pub mod transformer;

/*----- */
// Subscription enum inputs
/*----- */
#[derive(Debug, PartialEq, Hash, Eq, Clone, Copy)]
pub enum Exchange {
    BinanceSpot,
    PoloniexSpot,
}

impl fmt::Display for Exchange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Exchange::BinanceSpot => write!(f, "binancespot"), 
            Exchange::PoloniexSpot => write!(f, "poloniexspot"),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum StreamType {
    L1,
    L2,
    Trades,
}
impl fmt::Display for StreamType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            StreamType::L1 => write!(f, "l1"), 
            StreamType::L2 => write!(f, "l2"),
            StreamType::Trades => write!(f, "trades")
        }
    }
}

/*----- */
// Exchange subscription
/*----- */
#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct ExchangeSub {
    pub base: &'static str,
    pub quote: &'static str,
    pub stream_type: StreamType,
}

#[derive(Debug, PartialEq, Eq, Hash)]
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
