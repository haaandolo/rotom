pub mod exchange_connector;
pub mod protocols;
pub mod shared;
pub mod subscriber;

/*----- */
// Subscription enum inputs
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
pub struct ExchangeSub<'a> {
    pub base: &'a str,
    pub quote: &'a str,
    pub stream_type: StreamType,
}

impl<'a> ExchangeSub<'a> {
    pub fn new(_base: &'a str, _quote: &'a str, _stream_type: StreamType) -> Self {
        Self {
            base: _base,
            quote: _quote,
            stream_type: _stream_type,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Sub {
    pub exchange: ExchangeId,
    pub base: &'static str,
    pub quote: &'static str,
    pub stream_type: StreamType,
}

impl Sub {
    pub fn new(
        _exchange: ExchangeId,
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

    pub fn convert_subscription(self) -> ExchangeSub<'static> {
        ExchangeSub {
            base: self.base,
            quote: self.quote,
            stream_type: self.stream_type,
        }
    }
}

//////
pub struct ConnectorStuct<'a, ExchangeConnector> {
    pub connector: ExchangeConnector,
    pub subs: Vec<ExchangeSub<'a>>,
}
