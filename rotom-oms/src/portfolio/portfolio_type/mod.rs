use rotom_data::event_models::market_event::{DataKind, MarketEvent};
use rotom_strategy::{Signal, SignalForceExit};

use crate::{event::Event, execution::FillEvent};

use super::{error::PortfolioError, position::PositionUpdate, OrderEvent};

pub mod default_portfolio;

/*----- */
// Market Updater
/*----- */
pub trait MarketUpdater {
    fn update_from_market(
        &mut self,
        market: &MarketEvent<DataKind>,
    ) -> Result<Option<PositionUpdate>, PortfolioError>;
}

/*----- */
// Order Generator
/*----- */
pub trait OrderGenerator {
    fn generate_order(&mut self, signal: &Signal) -> Result<Option<OrderEvent>, PortfolioError>;

    fn generate_exit_order(
        &mut self,
        signal: SignalForceExit,
    ) -> Result<Option<OrderEvent>, PortfolioError>;
}

/*----- */
// Fill Updater
/*----- */
pub trait FillUpdater {
    fn update_from_fill(&mut self, fill: &FillEvent) -> Result<Vec<Event>, PortfolioError>;
}
