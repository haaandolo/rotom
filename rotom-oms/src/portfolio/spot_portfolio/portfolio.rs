use parking_lot::RwLock;
use std::{collections::HashMap, sync::Arc};

use rotom_data::{error::SocketError, shared::subscription_models::ExchangeId};

use crate::{
    exchange::{
        binance::binance_client::BinancePrivateData, poloniex::poloniex_client::PoloniexPrivateData,
    },
    execution_manager::builder::TraderId,
    model::{
        account_data::AccountDataBalance,
        balance::{Balance, SpotBalanceId},
        ClientOrderId,
    },
    portfolio::{allocator::spot_arb_allocator::SpotArbAllocator, position2::Position2},
};

#[derive(Debug)]
pub struct BalanceMap(pub HashMap<SpotBalanceId, Balance>);

impl BalanceMap {
    pub fn update_balance(&mut self, balance_id: SpotBalanceId) {
        todo!()
    }
}

#[derive(Debug)]
pub struct SpotPortfolio2 {
    pub balances: Arc<RwLock<BalanceMap>>,
    open_positions: HashMap<(TraderId, ClientOrderId), Position2>,
    allocator: SpotArbAllocator,
}

impl SpotPortfolio2 {
    pub async fn init(exchanges: Vec<ExchangeId>) -> Result<SpotPortfolio2, SocketError> {
        let mut balances = HashMap::new();

        for exchange in exchanges.iter() {
            let exchange_balance = match exchange {
                ExchangeId::BinanceSpot => {
                    let balance = BinancePrivateData::new().get_balance_all().await?;
                    let asset_balance: Vec<AccountDataBalance> = balance.into();
                    asset_balance
                }
                ExchangeId::PoloniexSpot => {
                    let balance = PoloniexPrivateData::new().get_balance_all().await?;
                    let asset_balance: Vec<AccountDataBalance> = balance.into();
                    asset_balance
                }
            };

            for asset_balance in exchange_balance.into_iter() {
                balances.insert(SpotBalanceId::from(&asset_balance), asset_balance.balance);
            }
        }

        Ok(SpotPortfolio2 {
            balances: Arc::new(RwLock::new(BalanceMap(balances))),
            open_positions: HashMap::with_capacity(100),
            allocator: SpotArbAllocator,
        })
    }
}
