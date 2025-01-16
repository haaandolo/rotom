use parking_lot::RwLock;
use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

use rotom_data::error::SocketError;

use crate::{
    exchange::ExecutionClient,
    execution_manager::builder::TraderId,
    model::{
        account_data::AccountDataBalance,
        balance::{Balance, SpotBalanceId},
        ClientOrderId,
    },
    portfolio::{allocator::spot_arb_allocator::SpotArbAllocator, position2::Position2},
};

/*----- */
// Balance Map
/*----- */
#[derive(Debug)]
pub struct BalanceMap(pub HashMap<SpotBalanceId, Balance>);

impl BalanceMap {
    pub fn update_balance(&mut self, balance_id: SpotBalanceId) {
        todo!()
    }
}

/*----- */
// Spot Portfolio
/*----- */
#[derive(Debug)]
pub struct SpotPortfolio2 {
    pub balances: Arc<RwLock<BalanceMap>>,
    open_positions: HashMap<(TraderId, ClientOrderId), Position2>,
    allocator: SpotArbAllocator,
}

impl SpotPortfolio2 {}

/*----- */
// Spot Portfolio Builder
/*----- */
type BalanceFutures =
    Pin<Box<dyn Future<Output = Result<Vec<AccountDataBalance>, SocketError>> + Send>>;

#[derive(Default)]
pub struct SpotPortfolio2Builder {
    balance_futures: Vec<BalanceFutures>,
}

impl SpotPortfolio2Builder {
    pub fn add_exchange<Exchange: ExecutionClient + 'static>(mut self) -> Self {
        self.balance_futures
            .push(Box::pin(Exchange::get_balances()));

        self
    }

    pub async fn build(self) -> Result<SpotPortfolio2, SocketError> {
        let mut balances = HashMap::new();
        let exchange_balances_joined = futures::future::try_join_all(self.balance_futures).await?;

        for asset_balance in exchange_balances_joined.into_iter().flatten() {
            balances.insert(SpotBalanceId::from(&asset_balance), asset_balance.balance);
        }

        Ok(SpotPortfolio2 {
            balances: Arc::new(RwLock::new(BalanceMap(balances))),
            open_positions: HashMap::with_capacity(100),
            allocator: SpotArbAllocator,
        })
    }
}
