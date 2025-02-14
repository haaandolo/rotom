use std::collections::HashMap;

use tracing::error;

use crate::{
    model::{
        balance::{Balance, SpotBalanceId},
        execution_response::{AccountBalance, AccountBalanceDelta},
        ClientOrderId,
    },
    portfolio::position2::Position2,
};

/*----- */
// Order Map
/*----- */
#[derive(Debug)]
pub struct OrderMap(pub HashMap<ClientOrderId, Position2>);

impl OrderMap {
    pub fn update_or_insert(&mut self) {
        todo!()
    }

    pub fn evaluate_order(&self) {
        todo!()
    }

    pub fn find_all_by_trader_id(&self) {
        todo!()
    }

    pub fn get_by_cid(&self) {
        todo!()
    }
}

/*----- */
// Balance Map
/*----- */
#[derive(Debug)]
pub struct BalanceMap(pub HashMap<SpotBalanceId, Balance>); // todo: change spot balanc eto balanceId

impl BalanceMap {
    pub fn update_balance(&mut self, balance_update: &AccountBalance) {
        let balance_id = SpotBalanceId::from(balance_update);
        match self.0.get_mut(&balance_id) {
            Some(balance) => {
                if balance.total != balance_update.balance.total {
                    balance.total = balance_update.balance.total;
                }
            }
            None => {
                let _ = self.0.insert(balance_id, balance_update.balance);
            }
        }
    }

    pub fn update_balance_delta(&mut self, balance_delta: &AccountBalanceDelta) {
        let balance_id = SpotBalanceId::from(balance_delta);
        match self.0.get_mut(&balance_id) {
            Some(balance) => {
                balance.total += balance_delta.total;
            }
            None => error!(
                message = "BalanceMap receieved a balance delta not in portfolio",
                balance_id  = %balance_id
            ),
        }
    }
}
