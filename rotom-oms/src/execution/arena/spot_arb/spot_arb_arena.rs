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
pub struct SpotArbArena<LiquidExchange, IlliquidExchange>
where
    LiquidExchange: ExecutionClient + 'static,
    IlliquidExchange: ExecutionClient + 'static,
{
    pub executor: SpotArbExecutor<LiquidExchange, IlliquidExchange>,
}

impl<LiquidExchange, IlliquidExchange> SpotArbArena<LiquidExchange, IlliquidExchange>
where
    LiquidExchange: ExecutionClient + 'static,
    IlliquidExchange: ExecutionClient + 'static,
{
    pub async fn init() -> Self {
        let executor = SpotArbExecutor::<LiquidExchange, IlliquidExchange>::init()
            .await
            .unwrap(); // todo
        Self { executor }
    }

    pub async fn testing(&mut self) {
        while let Some(msg) = self.executor.streams.recv().await {
            println!("{:#?}", msg)
        }
    }
}

/*----- */
// Impl FillGenerator for SpotArbArena
/*----- */
impl<LiquidExchange, IlliquidExchange> FillGenerator
    for SpotArbArena<LiquidExchange, IlliquidExchange>
where
    LiquidExchange: ExecutionClient,
    IlliquidExchange: ExecutionClient,
{
    fn generate_fill(&mut self, _order: OrderEvent) -> Result<FillEvent, ExecutionError> {
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
