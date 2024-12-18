pub mod error;
pub mod in_memory;
pub mod spot_in_memory;
pub mod redis;

use crate::model::balance::Balance;

use super::position::{Position, PositionId};
use error::RepositoryError;
use rotom_data::{Market, MarketId};
use uuid::Uuid;

/*----- */
// Position Handler
/*----- */
pub trait PositionHandler {
    fn set_open_position(&mut self, position: Position) -> Result<(), RepositoryError>;

    fn get_open_position(
        &mut self,
        position_id: &PositionId,
    ) -> Result<Option<Position>, RepositoryError>;

    fn get_open_positions<'a, Markets: Iterator<Item = &'a Market>>(
        &mut self,
        engine_id: Uuid,
        markets: Markets,
    ) -> Result<Vec<Position>, RepositoryError>;

    fn remove_position(
        &mut self,
        position_id: &PositionId,
    ) -> Result<Option<Position>, RepositoryError>;

    fn set_exited_position(
        &mut self,
        engine_id: Uuid,
        position: Position,
    ) -> Result<(), RepositoryError>;

    fn get_exited_positions(&mut self, engine_id: Uuid) -> Result<Vec<Position>, RepositoryError>;
}

/*----- */
// Balance Handler
/*----- */
pub trait BalanceHandler {
    fn set_balance(&mut self, engine_id: Uuid, balance: Balance) -> Result<(), RepositoryError>;

    fn get_balance(&mut self, engine_id: Uuid) -> Result<Balance, RepositoryError>;
}

/*----- */
// Statistics Handler
/*----- */
pub trait StatisticHandler<Statistic> {
    fn set_statistics(
        &mut self,
        market_id: MarketId,
        statistics: Statistic,
    ) -> Result<(), RepositoryError>;

    fn get_statistics(&mut self, market_id: &MarketId) -> Result<Statistic, RepositoryError>;
}

/*----- */
// Exited position
/*----- */
pub type ExitedPositionId = String;

pub fn determine_exited_positions_id(engine_id: Uuid) -> ExitedPositionId {
    format!("position_exited_{}", engine_id)
}
