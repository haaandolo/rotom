use error::RepositoryError;
use uuid::Uuid;

use super::{
    position::{Position, PositionId},
    Balance,
};

pub mod error;
pub mod in_memory;
pub mod redis;

/*----- */
// Position Handler
/*----- */
pub trait PositionHandler {
    fn set_open_position(&mut self, position: Position) -> Result<(), RepositoryError>;

    fn get_open_position(
        &mut self,
        position_id: &PositionId,
    ) -> Result<Option<Position>, RepositoryError>;

    // TODO: look at barter
    fn get_open_positions(
        &mut self,
        position_id: &PositionId,
    ) -> Result<Option<Position>, RepositoryError>;

    fn remove_positions(
        &mut self,
        position_id: &PositionId,
    ) -> Result<Option<Position>, RepositoryError>;

    fn set_exited_position(
        &mut self,
        engine_id: Uuid,
        position: Position,
    ) -> Result<(), RepositoryError>;

    fn get_exited_position(&mut self, engine_id: Uuid) -> Result<Vec<Position>, RepositoryError>;
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
pub trait StatisticsHandler<Statistic> {
    fn set_statistics(
        &mut self,
        market_id: String, // TODO: change to MarketId
        statistics: Statistic,
    ) -> Result<(), RepositoryError>;

    fn get_statistics(
        &mut self,
        market_id: String, // TODO: change to MarketId
    ) -> Result<Statistic, RepositoryError>;
}

pub type ExitedPositionId = String;

pub fn determine_exited_positions_id(engine_id: Uuid) -> ExitedPositionId {
    format!("position_exited_{}", engine_id)
}
