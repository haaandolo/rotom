use std::collections::HashMap;

use async_trait::async_trait;
use chrono::Utc;
use futures::stream::select_all;
use rotom_data::{
    shared::subscription_models::{ExchangeId, Instrument},
    MarketMeta,
};
use tokio::sync::mpsc::{self, unbounded_channel};
use tokio_stream::StreamMap;

use crate::{
    exchange::{
        binance::{binance_client::BinanceExecution, requests::user_data::BinanceUserData},
        poloniex::{poloniex_client::PoloniexExecution, requests::user_data::PoloniexUserData},
        spawn_ws_read, ExecutionClient2,
    },
    execution::{error::ExecutionError, Fees, FillEvent},
    portfolio::OrderEvent,
};

/*----- */
// Arb Trader Trait
/*----- */
#[async_trait]
pub trait ArbTraderArena {
    type ExchangeOne: ExecutionClient2;
    type ExchangeTwo: ExecutionClient2;

    async fn init() -> bool;

    fn generate_fill(&self, order: &OrderEvent) -> Result<FillEvent, ExecutionError>;
}

#[derive(Debug)]
enum CombinedUserStreams<UserDataOne, UserDataTwo> {
    ExchangeOne(UserDataOne),
    ExchangeTwo(UserDataTwo),
}

/*----- */
// Arb trader
/*----- */
pub struct ArbExecutor;

#[async_trait]
impl ArbTraderArena for ArbExecutor {
    type ExchangeOne = BinanceExecution;
    type ExchangeTwo = PoloniexExecution;

    async fn init() -> bool {
        let mut binance_execution = BinanceExecution::init().await.unwrap(); // todo
        let mut poloniex_excution = PoloniexExecution::init().await.unwrap(); // todo

        let (combined_tx, mut combined_rx) = mpsc::unbounded_channel();
        let combined_tx_cloned = combined_tx.clone();

        tokio::spawn(async move {
            while let Some(message) = binance_execution.rx.recv().await {
                let _ = combined_tx_cloned.send(CombinedUserStreams::ExchangeOne(message));
            }
        });

        tokio::spawn(async move {
            while let Some(message) = poloniex_excution.rx.recv().await {
                let _ = combined_tx
                    .clone()
                    .send(CombinedUserStreams::ExchangeTwo(message));
            }
        });

        while let Some(message) = combined_rx.recv().await {
            println!("{:#?}", message);
        }

        true
    }

    fn generate_fill(&self, order: &OrderEvent) -> Result<FillEvent, ExecutionError> {
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
