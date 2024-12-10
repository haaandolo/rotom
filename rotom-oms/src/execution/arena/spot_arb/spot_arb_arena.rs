use std::collections::HashMap;

use chrono::Utc;
use rotom_data::{
    shared::subscription_models::{ExchangeId, Instrument},
    Market, MarketMeta,
};
use tokio::sync::mpsc::{self, error::TryRecvError};

use crate::{
    exchange::{
        binance::binance_client::BinanceExecution, poloniex::poloniex_client::PoloniexExecution,
        ExecutionClient,
    },
    execution::{error::ExecutionError, Fees, FillEvent, FillGenerator},
    model::{
        account_data::AccountData,
        order::{ExecutionRequest, OrderEvent},
    },
};

/*----- */
// Spot Arbitrage Arena
/*----- */
pub struct SpotArbArena {
    // pub executor: HashMap<ExchangeId, Executors>,
    pub order_rx: mpsc::UnboundedReceiver<ExecutionRequest>,
    // pub trader_order_updater: HashMap<String, mpsc::Sender<AccountData>>,
}

impl SpotArbArena {
    pub async fn new(
        order_rx: mpsc::UnboundedReceiver<ExecutionRequest>,
        exchange_ids: Vec<ExchangeId>, // trader_order_updater: HashMap<String, mpsc::Sender<AccountData>>,
    ) -> Self {
        // //
        // let mut executors = HashMap::new();
        // for exchange in exchange_ids.into_iter() {
        //     match exchange {
        //         ExchangeId::BinanceSpot => {
        //             executors.insert(ExchangeId::BinanceSpot, BinanceExecution::new());
        //         }
        //         ExchangeId::PoloniexSpot => {
        //             executors.insert(ExchangeId::PoloniexSpot, PoloniexExecution::new());
        //         }
        //     }
        // }

        //

        Self {
            // executor,
            order_rx,
            // trader_order_updater,
        }
    }

    pub async fn testing(mut self) {
        // loop {
        //     match self.order_rx.try_recv() {
        //         Ok(order) => self.long_exchange_transfer(order).await,
        //         Err(TryRecvError::Empty) => tokio::task::yield_now().await,
        //         Err(TryRecvError::Disconnected) => break,
        //     }
        // }

        // while let Some(msg) = self.order_rx.recv().await {
        //     println!("in arena -> {:#?}", msg);
        // }

        // while let Some(msg) = self.executor.streams.recv().await {
        //     println!("{:#?}", msg)
        // }
    }
}
