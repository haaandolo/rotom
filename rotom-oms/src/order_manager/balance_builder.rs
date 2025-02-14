use std::{collections::HashMap, future::Future, pin::Pin};

use rotom_data::error::SocketError;

use crate::{
    exchange::ExecutionClient,
    model::{balance::SpotBalanceId, execution_response::AccountBalance},
};

use super::maps::BalanceMap;

type BalanceFutures =
    Pin<Box<dyn Future<Output = Result<Vec<AccountBalance>, SocketError>> + Send>>;

/*----- */
// Balance Builder
/*----- */
#[derive(Default)]
pub struct BalanceBuilder {
    balance_futures: Vec<BalanceFutures>,
}

impl BalanceBuilder {
    pub fn add_exchange<Exchange: ExecutionClient + 'static>(mut self) -> Self {
        self.balance_futures
            .push(Box::pin(Exchange::get_balances()));

        self
    }

    pub async fn build(self) -> Result<BalanceMap, SocketError> {
        let mut balances = HashMap::new();
        let exchange_balances_joined = futures::future::try_join_all(self.balance_futures).await?;

        for asset_balance in exchange_balances_joined.into_iter().flatten() {
            balances.insert(SpotBalanceId::from(&asset_balance), asset_balance.balance);
        }

        Ok(BalanceMap(balances))
    }
}
