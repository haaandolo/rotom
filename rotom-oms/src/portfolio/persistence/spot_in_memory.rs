use std::collections::HashMap;

use rotom_data::{ExchangeAssetId, Market};
use tracing::error;
use uuid::Uuid;

use crate::{
    model::{
        balance::{Balance, SpotBalanceId},
        execution_response::{AccountBalance, AccountBalanceDelta},
    },
    portfolio::{
        position::{determine_position_id, Position, PositionId},
        position2::Position2,
    },
};

use super::{determine_exited_positions_id, error::RepositoryError};

/*----- */
// InMemoryRepository
/*----- */
#[derive(Debug, Default)]
pub struct SpotInMemoryRepository {
    open_positions: HashMap<ExchangeAssetId, Position2>,
    closed_positions: HashMap<String, Vec<Position>>,
    current_balance: HashMap<SpotBalanceId, Balance>,
}

/*----- */
// Impl BalanceHandler for InMemoryRepository
/*----- */
impl SpotInMemoryRepository {
    // Balance Hanlder
    pub fn set_balance(
        &mut self,
        balance_id: SpotBalanceId,
        balance: Balance,
    ) -> Result<(), RepositoryError> {
        self.current_balance.insert(balance_id, balance);
        Ok(())
    }

    pub fn update_balance(&mut self, balance_update: &AccountBalance) {
        let balance_id = SpotBalanceId::from(balance_update);
        match self.current_balance.get_mut(&balance_id) {
            Some(balance) => {
                if balance.total != balance_update.balance.total {
                    balance.total = balance_update.balance.total;
                }
            }
            None => {
                let _ = self
                    .current_balance
                    .insert(balance_id, balance_update.balance);
            }
        }
    }

    pub fn update_balance_delta(&mut self, balance_delta: &AccountBalanceDelta) {
        let balance_id = SpotBalanceId::from(balance_delta);
        match self.current_balance.get_mut(&balance_id) {
            Some(balance) => {
                balance.total += balance_delta.total;
            }

            None => error!(
                message = "SpotInMemoryRepository receieved a balance delta not in portfolio",
                balance_id  = %balance_id
            ),
        }
    }

    pub fn get_balance(&mut self, balance_id: &SpotBalanceId) -> Result<Balance, RepositoryError> {
        self.current_balance
            .get(balance_id)
            .copied()
            .ok_or(RepositoryError::ExpectedDataNotPresentError)
    }

    // Position Handler
    pub fn set_open_position(&mut self, position: Position2) -> Result<(), RepositoryError> {
        self.open_positions
            .insert(position.position_id.clone(), position);
        Ok(())
    }

    pub fn get_open_position_mut(
        &mut self,
        position_id: &ExchangeAssetId,
    ) -> Result<Option<&mut Position2>, RepositoryError> {
        Ok(self.open_positions.get_mut(position_id))
    }

    pub fn get_open_position(
        &self,
        position_id: &ExchangeAssetId,
    ) -> Result<Option<&Position2>, RepositoryError> {
        Ok(self.open_positions.get(position_id))
    }

    // todo:
    // pub fn get_open_positions<'a, Markets: Iterator<Item = &'a Market>>(
    //     &mut self,
    //     engine_id: Uuid,
    //     markets: Markets,
    // ) -> Result<Vec<Position>, RepositoryError> {
    //     Ok(markets
    //         .filter_map(|market| {
    //             self.open_positions
    //                 .get(&determine_position_id(
    //                     engine_id,
    //                     &market.exchange,
    //                     &market.instrument,
    //                 ))
    //                 .cloned()
    //         })
    //         .collect())
    // }

    pub fn remove_position(
        &mut self,
        position_id: &ExchangeAssetId,
    ) -> Result<Option<Position2>, RepositoryError> {
        Ok(self.open_positions.remove(position_id))
    }

    pub fn set_exited_position(
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

    pub fn get_exited_positions(
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

/////////////////////////////////////////////////////////////
// impl<Statistic> InMemoryRepository2<Statistic>
// where
//     Statistic: PositionSummariser,
// {
//     pub fn new() -> Self {
//         Self {
//             open_positions: HashMap::new(),
//             closed_positions: HashMap::new(),
//             current_balance: HashMap::new(),
//             statistics: HashMap::new(),
//         }
//     }
// }
// /*----- */
// // Impl PositionHandler for InMemoryRepository
// /*----- */
// impl<Statistic> PositionHandler for InMemoryRepository2<Statistic>
// where
//     Statistic: PositionSummariser,
// {
//     fn set_open_position(&mut self, position: Position) -> Result<(), RepositoryError> {
//         self.open_positions
//             .insert(position.position_id.clone(), position);
//         Ok(())
//     }

//     fn get_open_positions<'a, Markets: Iterator<Item = &'a Market>>(
//         &mut self,
//         engine_id: Uuid,
//         markets: Markets,
//     ) -> Result<Vec<Position>, RepositoryError> {
//         Ok(markets
//             .filter_map(|market| {
//                 self.open_positions
//                     .get(&determine_position_id(
//                         engine_id,
//                         &market.exchange,
//                         &market.instrument,
//                     ))
//                     .cloned()
//             })
//             .collect())
//     }

//     fn remove_position(
//         &mut self,
//         position_id: &PositionId,
//     ) -> Result<Option<Position>, RepositoryError> {
//         Ok(self.open_positions.remove(position_id))
//     }

//     fn set_exited_position(
//         &mut self,
//         engine_id: uuid::Uuid,
//         position: Position,
//     ) -> Result<(), RepositoryError> {
//         let exited_positions_key = determine_exited_positions_id(engine_id);

//         match self.closed_positions.get_mut(&exited_positions_key) {
//             None => {
//                 self.closed_positions
//                     .insert(exited_positions_key, vec![position]);
//             }
//             Some(closed_positions) => closed_positions.push(position),
//         }

//         Ok(())
//     }

//     fn get_exited_positions(
//         &mut self,
//         engine_id: uuid::Uuid,
//     ) -> Result<Vec<Position>, RepositoryError> {
//         Ok(self
//             .closed_positions
//             .get(&determine_exited_positions_id(engine_id))
//             .cloned()
//             .unwrap_or_default())
//     }
// }

// /*----- */
// // Impl BalanceHandler for InMemoryRepository
// /*----- */
// impl<Statistic> InMemoryRepository2<Statistic>
// where
//     Statistic: PositionSummariser,
// {
//     pub fn set_balance(
//         &mut self,
//         exchange: ExchangeId,
//         asset: String,
//         balance:Balance,
//     ) -> Result<(), RepositoryError> {

//         self.current_balance
//             .insert(Balance::balance_id(engine_id), balance);
//         Ok(())
//     }

//     pub fn get_balance(&mut self, engine_id: Uuid) -> Result<Balance, RepositoryError> {
//         self.current_balance
//             .get(&Balance::balance_id(engine_id))
//             .copied()
//             .ok_or(RepositoryError::ExpectedDataNotPresentError)
//     }
// }

// /*----- */
// // Impl StatisticHandler for InMemoryRepository
// /*----- */
// impl<Statistic> StatisticHandler<Statistic> for InMemoryRepository2<Statistic>
// where
//     Statistic: PositionSummariser,
// {
//     fn set_statistics(
//         &mut self,
//         market_id: MarketId,
//         statistic: Statistic,
//     ) -> Result<(), RepositoryError> {
//         self.statistics.insert(market_id, statistic);
//         Ok(())
//     }

//     fn get_statistics(&mut self, market_id: &MarketId) -> Result<Statistic, RepositoryError> {
//         self.statistics
//             .get(market_id)
//             .copied()
//             .ok_or(RepositoryError::ExpectedDataNotPresentError)
//     }
// }
