use std::collections::HashMap;

use rotom_data::{Market, MarketId};
use uuid::Uuid;

use crate::{
    oms::{
        position::{determine_position_id, Position, PositionId},
        Balance, BalanceId,
    },
    statistic::summary::PositionSummariser,
};

use super::{
    determine_exited_positions_id, error::RepositoryError, BalanceHandler, PositionHandler,
    StatisticHandler,
};

/*----- */
// InMemoryRepository
/*----- */
#[derive(Debug, Default)]
pub struct InMemoryRepository<Statistic>
where
    Statistic: PositionSummariser,
{
    open_positions: HashMap<PositionId, Position>,
    closed_positions: HashMap<String, Vec<Position>>,
    current_balance: HashMap<BalanceId, Balance>,
    statistics: HashMap<MarketId, Statistic>,
}

impl<Statistic> InMemoryRepository<Statistic>
where
    Statistic: PositionSummariser,
{
    pub fn new() -> Self {
        Self {
            open_positions: HashMap::new(),
            closed_positions: HashMap::new(),
            current_balance: HashMap::new(),
            statistics: HashMap::new(),
        }
    }
}
/*----- */
// Impl PositionHandler for InMemoryRepository
/*----- */
impl<Statistic> PositionHandler for InMemoryRepository<Statistic>
where
    Statistic: PositionSummariser,
{
    fn set_open_position(&mut self, position: Position) -> Result<(), RepositoryError> {
        self.open_positions
            .insert(position.position_id.clone(), position);
        Ok(())
    }

    fn get_open_position(
        &mut self,
        position_id: &PositionId,
    ) -> Result<Option<Position>, RepositoryError> {
        Ok(self.open_positions.get(position_id).cloned())
    }

    fn get_open_positions<'a, Markets: Iterator<Item = &'a Market>>(
        &mut self,
        engine_id: Uuid,
        markets: Markets,
    ) -> Result<Vec<Position>, RepositoryError> {
        Ok(markets
            .filter_map(|market| {
                self.open_positions
                    .get(&determine_position_id(
                        engine_id,
                        &market.exchange,
                        &market.instrument,
                    ))
                    .cloned()
            })
            .collect())
    }

    fn remove_position(
        &mut self,
        position_id: &PositionId,
    ) -> Result<Option<Position>, RepositoryError> {
        Ok(self.open_positions.remove(position_id))
    }

    fn set_exited_position(
        &mut self,
        engine_id: uuid::Uuid,
        position: Position,
    ) -> Result<(), RepositoryError> {
        let exited_positions_key = determine_exited_positions_id(engine_id);

        match self.closed_positions.get_mut(&exited_positions_key) {
            None => {
                self.closed_positions
                    .insert(exited_positions_key, vec![position]);
            }
            Some(closed_positions) => closed_positions.push(position),
        }

        Ok(())
    }

    fn get_exited_positions(
        &mut self,
        engine_id: uuid::Uuid,
    ) -> Result<Vec<Position>, RepositoryError> {
        Ok(self
            .closed_positions
            .get(&determine_exited_positions_id(engine_id))
            .cloned()
            .unwrap_or_default())
    }
}

/*----- */
// Impl BalanceHandler for InMemoryRepository
/*----- */
impl<Statistic> BalanceHandler for InMemoryRepository<Statistic>
where
    Statistic: PositionSummariser,
{
    fn set_balance(&mut self, engine_id: Uuid, balance: Balance) -> Result<(), RepositoryError> {
        self.current_balance
            .insert(Balance::balance_id(engine_id), balance);
        Ok(())
    }

    fn get_balance(&mut self, engine_id: Uuid) -> Result<Balance, RepositoryError> {
        self.current_balance
            .get(&Balance::balance_id(engine_id))
            .copied()
            .ok_or(RepositoryError::ExpectedDataNotPresentError)
    }
}

/*----- */
// Impl StatisticHandler for InMemoryRepository
/*----- */
impl<Statistic> StatisticHandler<Statistic> for InMemoryRepository<Statistic>
where
    Statistic: PositionSummariser,
{
    fn set_statistics(
        &mut self,
        market_id: MarketId,
        statistic: Statistic,
    ) -> Result<(), RepositoryError> {
        self.statistics.insert(market_id, statistic);
        Ok(())
    }

    fn get_statistics(&mut self, market_id: &MarketId) -> Result<Statistic, RepositoryError> {
        self.statistics
            .get(market_id)
            .copied()
            .ok_or(RepositoryError::ExpectedDataNotPresentError)
    }
}
