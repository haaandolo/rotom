pub mod error;

use error::EngineError;
use parking_lot::Mutex;
use prettytable::Table;
use rotom_data::Market;
use rotom_oms::{
    execution_manager::builder::TraderId,
    portfolio::{
        persistence::{PositionHandler, StatisticHandler},
        portfolio_type::{FillUpdater, MarketUpdater, OrderGenerator},
        position::Position,
    },
    statistic::summary::{PositionSummariser, TableBuilder},
};
use serde::Serialize;
use std::{collections::HashMap, sync::Arc, thread};
use tokio::sync::{mpsc, oneshot};
use tracing::error;
use uuid::Uuid;

use crate::trader::TraderRun;

/*----- */
// Commands that can be sent trader
/*----- */
#[derive(Debug)]
pub enum Command {
    FetchOpenPositions(oneshot::Sender<Result<Vec<Position>, EngineError>>),
    Terminate(String),
    ExitAllPositions,
    ExitPosition(Market),
}

/*----- */
// Engine
/*----- */
pub struct Engine<Trader, Statistic, Portfolio>
where
    Trader: TraderRun,
    Statistic: Serialize + Send,
    Portfolio: PositionHandler
        + MarketUpdater
        + OrderGenerator
        + FillUpdater
        + StatisticHandler<Statistic>
        + Send
        + 'static,
{
    engine_id: Uuid,
    command_rx: mpsc::Receiver<Command>,
    portfolio: Arc<Mutex<Portfolio>>,
    traders: Vec<Trader>,
    trader_command_txs: HashMap<TraderId, mpsc::Sender<Command>>,
    statistics_summary: Statistic,
}

impl<Trader, Statistic, Portfolio> Engine<Trader, Statistic, Portfolio>
where
    Trader: TraderRun + Send + 'static,
    Statistic: PositionSummariser + TableBuilder + Serialize + Send + 'static,
    Portfolio: PositionHandler
        + MarketUpdater
        + OrderGenerator
        + FillUpdater
        + StatisticHandler<Statistic>
        + Send
        + 'static,
{
    /// Builder to construct [`Engine`] instances.
    pub fn builder() -> EngineBuilder<Trader, Statistic, Portfolio> {
        EngineBuilder::new()
    }

    // Runs each [`Trader`] it's own thread. Sends a message on the returned `mpsc::Receiver<bool>`
    // if all the [`Trader`]s have stopped organically (eg/ due to a finished [`MarketEvent`] feed).
    async fn run_traders(&mut self) -> mpsc::Receiver<bool> {
        // Extract Traders out of the Engine so we can move them into threads
        let traders = std::mem::take(&mut self.traders);

        // Run each Trader instance on it's own thread
        let mut thread_handles = Vec::with_capacity(traders.len());
        for trader in traders.into_iter() {
            let handle = thread::spawn(move || trader.run());
            thread_handles.push(handle);
        }

        // Create channel to notify the Engine when the Traders have stopped organically
        let (notify_tx, notify_rx) = mpsc::channel(1);

        // Create Task that notifies Engine when the Traders have stopped organically
        tokio::spawn(async move {
            for handle in thread_handles {
                if let Err(err) = handle.join() {
                    error!(
                        error = &*format!("{:?}", err),
                        "Trader thread has panicked during execution",
                    )
                }
            }

            let _ = notify_tx.send(true).await;
        });

        notify_rx
    }

    // Run the trading [`Engine`]. Spawns a thread for each [`Trader`] to run on. Asynchronously
    // receives [`Command`]s via the `command_rx` and actions them
    // (eg/ terminate_traders, fetch_open_positions). If all of the [`Trader`]s stop organically
    // (eg/ due to a finished [`MarketGenerator`]), the [`Engine`] terminates & prints a summary
    // for the trading session.
    pub async fn run(mut self) {
        // Run Traders on threads & send notification when they have stopped organically
        let mut notify_traders_stopped = self.run_traders().await;

        loop {
            // Action received commands from remote, or wait for all Traders to stop organically
            tokio::select! {
                _ = notify_traders_stopped.recv() => {
                    break;
                },

                command = self.command_rx.recv() => {
                    if let Some(command) = command {
                        match command {
                            Command::FetchOpenPositions(positions_tx) => {
                                self.fetch_open_positions(positions_tx).await;
                            },
                            Command::Terminate(message) => {
                                self.terminate_traders(message).await;
                                break;
                            },
                            Command::ExitPosition(market) => {
                                self.exit_position(market).await;
                            },
                            Command::ExitAllPositions => {
                                self.exit_all_positions().await;
                            },
                        }
                    } else {
                        // Terminate traders due to dropped receiver
                        break;
                    }
                }
            }
        }

        // Print Trading Session Summary
        self.generate_session_summary().printstd();
    }

    /// Fetches all the [`Engine`]'s open [`Position`]s and sends them on the provided
    /// `oneshot::Sender`.
    async fn fetch_open_positions(
        &self,
        positions_tx: oneshot::Sender<Result<Vec<Position>, EngineError>>,
    ) {
        // let open_positions = self
        //     .portfolio
        //     .lock()
        //     .get_open_positions(self.engine_id, self.trader_command_txs.keys())
        //     .map_err(EngineError::RepositoryInteractionError);

        // if positions_tx.send(open_positions).is_err() {
        //     warn!(
        //         why = "oneshot receiver dropped",
        //         "cannot action Command::FetchOpenPositions"
        //     );
        // }
    }

    /// Terminate every running [`Trader`] associated with this [`Engine`].
    async fn terminate_traders(&self, message: String) {
        // Firstly, exit all Positions
        self.exit_all_positions().await;
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        // Distribute Command::Terminate to all the Engine's Traders
        for (market, command_tx) in self.trader_command_txs.iter() {
            if command_tx
                .send(Command::Terminate(message.clone()))
                .await
                .is_err()
            {
                error!(
                    market = &*format!("{:?}", market),
                    why = "dropped receiver",
                    "failed to send Command::Terminate to Trader command_rx"
                );
            }
        }
    }

    /// Exit every open [`Position`] associated with this [`Engine`].
    async fn exit_all_positions(&self) {
        // for (market, command_tx) in self.trader_command_txs.iter() {
        //     if command_tx
        //         .send(Command::ExitPosition(market.clone()))
        //         .await
        //         .is_err()
        //     {
        //         error!(
        //             market = &*format!("{:?}", market),
        //             why = "dropped receiver",
        //             "failed to send Command::Terminate to Trader command_rx"
        //         );
        //     }
        // }
    }

    /// Exit a [`Position`]. Uses the [`Market`] provided to route this [`Command`] to the relevant
    /// [`Trader`] instance.
    async fn exit_position(&self, market: Market) {
        // if let Some((market_ref, command_tx)) = self.trader_command_txs.get_key_value(&market) {
        //     if command_tx
        //         .send(Command::ExitPosition(market))
        //         .await
        //         .is_err()
        //     {
        //         error!(
        //             market = &*format!("{:?}", market_ref),
        //             why = "dropped receiver",
        //             "failed to send Command::Terminate to Trader command_rx"
        //         );
        //     }
        // } else {
        //     warn!(
        //         market = &*format!("{:?}", market),
        //         why = "Engine has no trader_command_tx associated with provided Market",
        //         "failed to exit Position"
        //     );
        // }
    }

    /// Generate a trading session summary. Uses the Portfolio's statistics per [`Market`] in
    /// combination with the average statistics across all [`Market`]s traded.
    fn generate_session_summary(mut self) -> Table {
        todo!()
        // // Fetch statistics for each Market
        // let stats_per_market = self.trader_command_txs.into_keys().filter_map(|market| {
        //     let market_id = MarketId::from(&market);

        //     match self.portfolio.lock().get_statistics(&market_id) {
        //         Ok(statistics) => Some((market_id.0, statistics)),
        //         Err(error) => {
        //             error!(
        //                 ?error,
        //                 ?market,
        //                 "failed to get Market statistics when generating trading session summary"
        //             );
        //             None
        //         }
        //     }
        // });

        // // Generate average statistics across all markets using session's exited Positions
        // self.portfolio
        //     .lock()
        //     .get_exited_positions(self.engine_id)
        //     .map(|exited_positions| {
        //         self.statistics_summary.generate_summary(&exited_positions);
        //     })
        //     .unwrap_or_else(|error| {
        //         warn!(
        //             ?error,
        //             why = "failed to get exited Positions from Portfolio's repository",
        //             "failed to generate Statistics summary for trading session"
        //         );
        //     });

        // // Combine Total & Per-Market Statistics Into Table
        // summary::combine(stats_per_market.chain([("Total".to_owned(), self.statistics_summary)]))
    }
}

/*------ */
// Engine Builder
/*------ */
#[derive(Debug, Default)]
pub struct EngineBuilder<Trader, Statistic, Portfolio>
where
    Trader: TraderRun,
    Statistic: Serialize + Send,
    Portfolio: MarketUpdater + OrderGenerator + FillUpdater + Send,
{
    engine_id: Option<Uuid>,
    command_rx: Option<mpsc::Receiver<Command>>,
    portfolio: Option<Arc<Mutex<Portfolio>>>,
    traders: Option<Vec<Trader>>,
    trader_command_txs: Option<HashMap<TraderId, mpsc::Sender<Command>>>,
    statistics_summary: Option<Statistic>,
}

impl<Trader, Statistic, Portfolio> EngineBuilder<Trader, Statistic, Portfolio>
where
    Trader: TraderRun,
    Statistic: PositionSummariser + Serialize + Send,
    Portfolio: PositionHandler
        + StatisticHandler<Statistic>
        + MarketUpdater
        + OrderGenerator
        + FillUpdater
        + Send,
{
    fn new() -> Self {
        Self {
            engine_id: None,
            command_rx: None,
            portfolio: None,
            traders: None,
            trader_command_txs: None,
            statistics_summary: None,
        }
    }

    pub fn engine_id(self, value: Uuid) -> Self {
        Self {
            engine_id: Some(value),
            ..self
        }
    }

    pub fn command_rx(self, value: mpsc::Receiver<Command>) -> Self {
        Self {
            command_rx: Some(value),
            ..self
        }
    }

    pub fn portfolio(self, value: Arc<Mutex<Portfolio>>) -> Self {
        Self {
            portfolio: Some(value),
            ..self
        }
    }

    pub fn traders(self, value: Vec<Trader>) -> Self {
        Self {
            traders: Some(value),
            ..self
        }
    }

    pub fn trader_command_txs(self, value: HashMap<TraderId, mpsc::Sender<Command>>) -> Self {
        Self {
            trader_command_txs: Some(value),
            ..self
        }
    }

    pub fn statistics_summary(self, value: Statistic) -> Self {
        Self {
            statistics_summary: Some(value),
            ..self
        }
    }

    pub fn build(self) -> Result<Engine<Trader, Statistic, Portfolio>, EngineError> {
        Ok(Engine {
            engine_id: self
                .engine_id
                .ok_or(EngineError::BuilderIncomplete("engine_id"))?,
            command_rx: self
                .command_rx
                .ok_or(EngineError::BuilderIncomplete("command_rx"))?,
            portfolio: self
                .portfolio
                .ok_or(EngineError::BuilderIncomplete("portfolio"))?,
            traders: self
                .traders
                .ok_or(EngineError::BuilderIncomplete("traders"))?,
            trader_command_txs: self
                .trader_command_txs
                .ok_or(EngineError::BuilderIncomplete("trader_command_txs"))?,
            statistics_summary: self
                .statistics_summary
                .ok_or(EngineError::BuilderIncomplete("statistics_summary"))?,
        })
    }
}
