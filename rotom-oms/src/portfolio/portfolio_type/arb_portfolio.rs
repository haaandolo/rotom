use rotom_data::{error::SocketError, exchange, shared::subscription_models::ExchangeId};

use crate::{
    exchange::{
        binance::binance_client::BinancePrivateData, poloniex::poloniex_client::PoloniexPrivateData,
    },
    portfolio::{persistence::in_memory2::InMemoryRepository2, AssetBalance, BalanceId2},
};

#[derive(Debug)]
pub struct ArbPortfolio {
    pub exchanges: Vec<ExchangeId>,
    pub repository: InMemoryRepository2,
}

impl ArbPortfolio {
    pub fn new(exchanges: Vec<ExchangeId>, repository: InMemoryRepository2) -> Self {
        Self {
            exchanges,
            repository,
        }
    }

    pub async fn init(&mut self) -> Result<(), SocketError> {
        for exchange in self.exchanges.iter() {
            let exchange_balance = match exchange {
                ExchangeId::BinanceSpot => {
                    let balance = BinancePrivateData::new().get_balance_all().await?;
                    let asset_balance: Vec<AssetBalance> = balance.into();
                    asset_balance
                }
                ExchangeId::PoloniexSpot => {
                    let balance = PoloniexPrivateData::new().get_balance_all().await?;
                    let asset_balance: Vec<AssetBalance> = balance.into();
                    asset_balance
                }
            };
            for asset_balance in exchange_balance.into_iter() {
                self.repository
                    .set_balance(BalanceId2::from(&asset_balance), asset_balance.balance)
                    .unwrap(); // todo
            }
        }

        println!("portfolio --> {:#?}", self.repository);

        Ok(())
    }

    pub fn bootstrap_repository(&mut self, balances: Vec<AssetBalance>) {
        // self.repository.set
    }
}
