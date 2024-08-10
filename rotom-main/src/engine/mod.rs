pub mod error;

use error::EngineError;
use rotom_data::event_models::market_event::{DataKind, MarketEvent};

use crate::data::MarketGenerator;

/*----- */
// Trader Lego
/*----- */
pub struct TraderLego<Data>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
{
    pub data: Data,
}

/*----- */
// Trader
/*----- */
pub struct Trader<Data>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
{
    pub data: Data,
}

impl<Data> Trader<Data>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
{
    pub fn new(lego: TraderLego<Data>) -> Self {
        Self { data: lego.data }
    }

    pub fn builder() -> TraderBuilder<Data> {
        TraderBuilder::new()
    }

    pub fn run(mut self) {
        loop {
           let market_data = self.data.next();
           println!("{:?}", market_data)
        }
    }

}

/*----- */
// Trader builder
/*----- */
#[derive(Debug, Default)]
pub struct TraderBuilder<Data>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
{
    pub data: Option<Data>,
}

impl<Data> TraderBuilder<Data>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
{
    pub fn new() -> Self {
        Self { data: None }
    }

    pub fn data(self, value: Data) -> Self {
        Self {
            data: Some(value),
            // ..self
        }
    }

    pub fn build(self) -> Result<Trader<Data>, EngineError> {
        Ok(Trader {
            data: self.data.ok_or(EngineError::BuilderIncomplete("data"))?,
        })
    }
}
