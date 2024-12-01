use std::collections::HashMap;

use chrono::Utc;
use rotom_data::{
    shared::subscription_models::{ExchangeId, Instrument},
    Market, MarketMeta,
};
use tokio::sync::mpsc::{self, error::TryRecvError};

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
    pub order_rx: mpsc::UnboundedReceiver<OrderEvent>,
    pub trader_order_updater: HashMap<Market, mpsc::Sender<FillEvent>>,
    pub orders: HashMap<Market, OrderEvent>,
}

impl<LiquidExchange, IlliquidExchange> SpotArbArena<LiquidExchange, IlliquidExchange>
where
    LiquidExchange: ExecutionClient + 'static,
    IlliquidExchange: ExecutionClient + 'static,
{
    pub async fn new(
        order_rx: mpsc::UnboundedReceiver<OrderEvent>,
        trader_order_updater: HashMap<Market, mpsc::Sender<FillEvent>>,
    ) -> Self {
        let executor = SpotArbExecutor::<LiquidExchange, IlliquidExchange>::init()
            .await
            .unwrap(); // todo
        Self {
            executor,
            order_rx,
            trader_order_updater,
            orders: HashMap::new(),
        }
    }

    pub async fn long_exchange_transfer(&self, order: OrderEvent) {
        // Post market order
        let long = self.executor.liquid_exchange.open_order(order).await;
    }

    pub async fn testing(&mut self) {
        loop {
            match self.order_rx.try_recv() {
                Ok(order) => self.long_exchange_transfer(order).await,
                Err(TryRecvError::Empty) => tokio::task::yield_now().await,
                Err(TryRecvError::Disconnected) => break,
            }
        }
        // while let Some(msg) = self.order_rx.recv().await {
        //     println!("in arena -> {:#?}", msg);
        // }

        // while let Some(msg) = self.executor.streams.recv().await {
        //     println!("{:#?}", msg)
        // }
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
