use async_trait::async_trait;
use chrono::Utc;
use rotom_data::{
    error::SocketError,
    shared::subscription_models::{ExchangeId, Instrument},
    MarketMeta,
};
use tokio::sync::mpsc;

use crate::{
    exchange::{
        binance::binance_client::BinanceExecution, consume_account_data_ws,
        poloniex::poloniex_client::PoloniexExecution, ExecutionClient2,
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

    async fn init() -> Result<(), SocketError>;

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
#[derive(Debug, Default)]
pub struct ArbExecutor;

#[async_trait]
impl ArbTraderArena for ArbExecutor {
    type ExchangeOne = BinanceExecution;
    type ExchangeTwo = PoloniexExecution;

    async fn init() -> Result<(), SocketError> {
        // Convert first exchange ws to channel
        let (exchange_one_tx, mut exchange_one_rx) = mpsc::unbounded_channel();
        tokio::spawn(consume_account_data_ws::<BinanceExecution>(exchange_one_tx));

        // Convert second exchange ws to channel
        let (exchange_two_tx, mut exchange_two_rx) = mpsc::unbounded_channel();
        tokio::spawn(consume_account_data_ws::<PoloniexExecution>(
            exchange_two_tx,
        ));

        // Combine channels into one
        let (combined_tx, mut combined_rx) = mpsc::unbounded_channel();
        let combined_tx_cloned = combined_tx.clone();
        tokio::spawn(async move {
            while let Some(message) = exchange_one_rx.recv().await {
                let _ = combined_tx_cloned.send(CombinedUserStreams::ExchangeOne(message));
            }
        });

        tokio::spawn(async move {
            while let Some(message) = exchange_two_rx.recv().await {
                let _ = combined_tx.send(CombinedUserStreams::ExchangeTwo(message));
            }
        });

        while let Some(message) = combined_rx.recv().await {
            println!("{:#?}", message);
        }

        Ok(())
    }

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
