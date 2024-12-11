use std::collections::HashMap;

use chrono::Utc;
use rotom_data::{
    shared::subscription_models::{ExchangeId, Instrument},
    Market, MarketMeta,
};
use tokio::sync::mpsc::{self, error::TryRecvError};

use crate::{
    exchange::{
        binance::binance_client::BinanceExecution, consume_account_data_stream,
        poloniex::poloniex_client::PoloniexExecution, send_account_data_to_traders,
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
    pub order_rx: mpsc::UnboundedReceiver<ExecutionRequest>,
}

impl SpotArbArena {
    pub async fn init(
        order_rx: mpsc::UnboundedReceiver<ExecutionRequest>,
        trader_order_updater: HashMap<String, mpsc::Sender<AccountData>>,
        exchange_ids: Vec<ExchangeId>,
    ) -> Self {
        // Combine user data streams from different exchanges into one
        let (account_data_tx, account_data_rx) = mpsc::unbounded_channel();
        for exchange in exchange_ids.into_iter() {
            match exchange {
                ExchangeId::BinanceSpot => {
                    tokio::spawn(consume_account_data_stream::<BinanceExecution>(
                        account_data_tx.clone(),
                    ));
                }
                ExchangeId::PoloniexSpot => {
                    tokio::spawn(consume_account_data_stream::<PoloniexExecution>(
                        account_data_tx.clone(),
                    ));
                }
            }
        }

        // Send order updates to corresponding trader pair for exchange
        tokio::spawn(send_account_data_to_traders(
            trader_order_updater,
            account_data_rx,
        ));

        Self { order_rx }
    }
}

/////////
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
