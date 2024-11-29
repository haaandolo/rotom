use chrono::Utc;
use rotom_data::{
    shared::subscription_models::{ExchangeId, Instrument},
    MarketMeta,
};

use crate::{
    exchange::ExecutionClient,
    execution::{error::ExecutionError, Fees, FillEvent, FillGenerator},
    model::order::OrderEvent,
};

use super::spot_arb_executor::SpotArbExecutor;

/*----- */
// Spot Arbitrage Arena
/*----- */
pub struct SpotArbArena<ExchangeOne, ExchangeTwo>
where
    ExchangeOne: ExecutionClient + 'static,
    ExchangeTwo: ExecutionClient + 'static,
{
    pub executor: SpotArbExecutor<ExchangeOne, ExchangeTwo>,
}

impl<ExchangeOne, ExchangeTwo> SpotArbArena<ExchangeOne, ExchangeTwo>
where
    ExchangeOne: ExecutionClient + 'static,
    ExchangeTwo: ExecutionClient + 'static,
{
    pub async fn new() -> Self {
        let executor = SpotArbExecutor::<ExchangeOne, ExchangeTwo>::init()
            .await
            .unwrap(); // todo
        Self { executor }
    }
}

/*----- */
// Impl FillGenerator for SpotArbArena
/*----- */
impl<ExchangeOne, ExchangeTwo> FillGenerator for SpotArbArena<ExchangeOne, ExchangeTwo>
where
    ExchangeOne: ExecutionClient,
    ExchangeTwo: ExecutionClient,
{
    fn generate_fill(&self, _order: &OrderEvent) -> Result<FillEvent, ExecutionError> {
        Ok(FillEvent {
            time: Utc::now(),
            exchange: ExchangeId::BinanceSpot,
            instrument: Instrument {
                base: "op".to_string(),
                quote: "usdt".to_string(),
            },
            market_meta: MarketMeta {
                time: Utc::now(),
                close: 0.0,
            },
            decision: rotom_strategy::Decision::Long,
            quantity: 0.0,
            fill_value_gross: 0.0,
            fees: Fees {
                exchange: 0.0,
                network: 0.0,
                slippage: 0.0,
            },
        })
    }
}
