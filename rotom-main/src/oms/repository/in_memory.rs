use std::collections::HashMap;

use uuid::Uuid;

use crate::oms::{
    position::{Position, PositionId},
    Balance, BalanceId,
};

use super::{determine_exited_positions_id, error::RepositoryError, BalanceHandler, PositionHandler};


/*----- */
// InMemoryRepository
/*----- */
#[derive(Debug, Default)]
pub struct InMemoryRepository {
    open_positions: HashMap<PositionId, Position>,
    closed_positions: HashMap<String, Vec<Position>>,
    current_balance: HashMap<BalanceId, Balance>, // TODO: Statistics
}

/*----- */
// Impl PositionHandler for InMemoryRepository
/*----- */
impl PositionHandler for InMemoryRepository {
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

    // TODO: look at barter
    fn get_open_positions(
        &mut self,
        position_id: PositionId,
    ) -> Result<Option<Position>, RepositoryError> {
        Ok(None)
    }

    fn remove_positions(
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

    fn get_exited_position(
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
impl BalanceHandler for InMemoryRepository {
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

// TODO: Statistics