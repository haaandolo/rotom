pub mod assets;
pub mod error;
pub mod exchange;
pub mod metric;
pub mod model;
pub mod protocols;
pub mod shared;
pub mod streams;
pub mod temp;
pub mod transformer;

use std::fmt::Display;

use chrono::{DateTime, Utc};
use exchange::Identifier;
use serde::{Deserialize, Serialize};
use shared::subscription_models::{ExchangeId, Instrument};
use tokio::sync::mpsc::{self, UnboundedReceiver};

/*----- */
// MarketGenerator
/*----- */
pub trait MarketGenerator<Event> {
    fn next(&mut self) -> Feed<Event>;
}

/*----- */
// Feed
/*----- */
#[derive(Debug)]
pub enum Feed<Event> {
    Next(Event),
    UnHealthy,
    Finished,
}

/*----- */
// Market metadata
/*----- */
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct MarketMeta {
    pub close: f64, // todo change to price
    pub time: DateTime<Utc>,
}

impl Default for MarketMeta {
    fn default() -> Self {
        Self {
            close: 50000.0,
            time: Utc::now(),
        }
    }
}

/*----- */
// Asset Formatted - asset formatted corresponding to exchange
/*----- */
#[derive(Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct AssetFormatted(pub String); // smol str

impl From<(&ExchangeId, &Instrument)> for AssetFormatted {
    fn from((exchange, instrument): (&ExchangeId, &Instrument)) -> Self {
        match exchange {
            ExchangeId::BinanceSpot => {
                AssetFormatted(format!("{}{}", instrument.base, instrument.quote).to_uppercase())
            }
            ExchangeId::PoloniexSpot => {
                AssetFormatted(format!("{}_{}", instrument.base, instrument.quote).to_uppercase())
            }
            ExchangeId::HtxSpot => {
                AssetFormatted(format!("{}{}", instrument.base, instrument.quote))
            }
            ExchangeId::WooxSpot => AssetFormatted(
                format!("SPOT_{}_{}", instrument.base, instrument.quote).to_uppercase(),
            ),
            ExchangeId::BitstampSpot => {
                AssetFormatted(format!("{}{}", instrument.base, instrument.quote))
            }
        }
    }
}

/*----- */
// Asset & Exchange formatted - usually used for keys of hashmaps e.g BINANCESPOT_OPUSDT
/*----- */
#[derive(Debug, Eq, PartialEq, PartialOrd, Hash, Serialize, Deserialize, Clone)]
pub struct ExchangeAssetId(pub String); // smol str

// Below from is usually used to get the exchange & asset id when the asset is already formatted.
// For example, when asset is received straight out of the websocket
impl From<(&ExchangeId, &AssetFormatted)> for ExchangeAssetId {
    fn from((exchange, asset_formatted): (&ExchangeId, &AssetFormatted)) -> Self {
        ExchangeAssetId(format!("{}_{}", exchange.as_str(), asset_formatted.0).to_uppercase())
    }
}

impl From<(&ExchangeId, &Instrument)> for ExchangeAssetId {
    fn from((exchange, instrument): (&ExchangeId, &Instrument)) -> Self {
        match exchange {
            ExchangeId::BinanceSpot => ExchangeAssetId(
                format!(
                    "{}_{}{}",
                    exchange.as_str(),
                    instrument.base,
                    instrument.quote
                )
                .to_uppercase(),
            ),
            ExchangeId::PoloniexSpot => ExchangeAssetId(
                format!(
                    "{}_{}_{}",
                    exchange.as_str(),
                    instrument.base,
                    instrument.quote
                )
                .to_uppercase(),
            ),
            ExchangeId::HtxSpot => ExchangeAssetId(format!(
                "{}_{}{}",
                exchange.as_str(),
                instrument.base,
                instrument.quote
            )),
            ExchangeId::WooxSpot => ExchangeAssetId(format!(
                "{}_SPOT_{}_{}",
                exchange.as_str(),
                instrument.base.to_uppercase(),
                instrument.quote.to_uppercase()
            )),
            ExchangeId::BitstampSpot => ExchangeAssetId(format!(
                "{}_{}{}",
                exchange.as_str(),
                instrument.base,
                instrument.quote
            )),
        }
    }
}

/*----- */
// Markets
/*----- */
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Market {
    pub exchange: ExchangeId,
    pub instrument: Instrument,
}

impl Market {
    pub fn new(exchange: ExchangeId, instrument: Instrument) -> Self {
        Self {
            exchange,
            instrument,
        }
    }
}

impl Identifier<AssetFormatted> for Market {
    fn id(&self) -> AssetFormatted {
        AssetFormatted::from((&self.exchange, &self.instrument))
    }
}

impl From<(ExchangeId, Instrument)> for Market {
    fn from((exchange, instrument): (ExchangeId, Instrument)) -> Self {
        Self {
            exchange,
            instrument,
        }
    }
}

impl Display for Market {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "
                exchange: {}, 
                base: {}, 
                quote: {}
            ",
            self.exchange, self.instrument.base, self.instrument.quote
        )
    }
}

/*----- */
// Market ID
/*----- */
#[derive(Debug, Eq, Hash, PartialEq)]
pub struct MarketId(pub String);

impl MarketId {
    pub fn new(exchange: &ExchangeId, instrument: &Instrument) -> MarketId {
        MarketId(format!(
            "{}_{}_{}",
            exchange, instrument.base, instrument.quote
        ))
    }
}

impl From<&Market> for MarketId {
    fn from(market: &Market) -> MarketId {
        MarketId(format!(
            "{}_{}_{}",
            market.exchange, market.instrument.base, market.instrument.quote
        ))
    }
}

/*----- */
// Market feed
/*----- */
#[derive(Debug)]
pub struct MarketFeed<Event> {
    pub market_rx: UnboundedReceiver<Event>,
}

impl<Event> MarketFeed<Event> {
    pub fn new(market_rx: UnboundedReceiver<Event>) -> Self {
        Self { market_rx }
    }
}

/*----- */
// Impl MarketGenerator
/*----- */
impl<Event> MarketGenerator<Event> for MarketFeed<Event> {
    fn next(&mut self) -> Feed<Event> {
        loop {
            match self.market_rx.try_recv() {
                Ok(event) => break Feed::Next(event),
                Err(mpsc::error::TryRecvError::Empty) => continue,
                Err(mpsc::error::TryRecvError::Disconnected) => break Feed::Finished,
            }
        }
    }
}
