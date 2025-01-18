use std::collections::HashMap;

use futures::StreamExt;
use rotom_data::{
    model::market_event::{DataKind, MarketEvent},
    shared::subscription_models::{Instrument, StreamKind},
    streams::builder::dynamic,
    MarketFeed,
};
use rotom_oms::{
    exchange::ExecutionClient, execution_manager::builder::TraderId,
    model::{execution_response::ExecutionResponse, execution_request::ExecutionRequest},
};
use tokio::sync::mpsc::{self, Sender, UnboundedReceiver};
use uuid::Uuid;

use crate::engine::Command;

use super::{
    generate_decision::SpotArbOrderGenerator,
    trader::{SpotArbTrader, SpotArbTraderExecutionState, SpotArbTraderMetaData},
};

#[derive(Debug)]
pub struct SpotArbTradersBuilder {
    // Vec of traders to go into the engine
    pub traders: Vec<SpotArbTrader>,
    // Channel to send ExecutionRequest to OrderManagementSystem
    pub execution_request_tx: mpsc::UnboundedSender<ExecutionRequest>,
    // Engine can send remote command's to the trader
    pub engine_command_tx: HashMap<TraderId, Sender<Command>>,
    // Channels to send back ExecuionResponses back to Traders
    pub execution_response_txs: HashMap<TraderId, mpsc::UnboundedSender<ExecutionResponse>>,
}

impl SpotArbTradersBuilder {
    pub fn new(execution_request_tx: mpsc::UnboundedSender<ExecutionRequest>) -> Self {
        Self {
            traders: Vec::new(),
            execution_request_tx,
            engine_command_tx: HashMap::new(),
            execution_response_txs: HashMap::new(),
        }
    }

    pub async fn add_traders<LiquidExchange, IlliquidExchange>(
        mut self,
        markets: Vec<Instrument>,
    ) -> Self
    where
        LiquidExchange: ExecutionClient,
        IlliquidExchange: ExecutionClient,
    {
        for market in markets.into_iter() {
            // Initalise trader id and channels
            let trader_id = TraderId(Uuid::new_v4());
            let (engine_command_tx, engine_command_rx) = mpsc::channel(3);
            let (execution_response_tx, execution_response_rx) = mpsc::unbounded_channel();

            // Insert trader tx to receive remote commands
            self.engine_command_tx.insert(trader_id, engine_command_tx);

            // Insert execution response channels to hashmap
            self.execution_response_txs
                .insert(trader_id, execution_response_tx);

            // Build the trader
            let arb_trader = SpotArbTrader::builder()
                .engine_id(Uuid::new_v4())
                .trader_id(trader_id)
                .command_rx(engine_command_rx)
                .data(MarketFeed::new(
                    stream_trades::<LiquidExchange, IlliquidExchange>(&market).await,
                ))
                .execution_request_tx(self.execution_request_tx.clone())
                .execution_response_rx(execution_response_rx)
                .order_generator(SpotArbOrderGenerator::default())
                .meta_data(SpotArbTraderMetaData {
                    order: None,
                    execution_state: SpotArbTraderExecutionState::NoPosition,
                    liquid_exchange: LiquidExchange::CLIENT,
                    illiquid_exchange: IlliquidExchange::CLIENT,
                    market,
                })
                .build()
                .unwrap();

            self.traders.push(arb_trader);
        }

        self
    }

    #[allow(clippy::type_complexity)]
    pub fn build(
        self,
    ) -> (
        Vec<SpotArbTrader>,
        HashMap<TraderId, Sender<Command>>,
        HashMap<TraderId, mpsc::UnboundedSender<ExecutionResponse>>,
    ) {
        (
            self.traders,
            self.engine_command_tx,
            self.execution_response_txs,
        )
    }
}

// todo: make this into a spot arb specfic stream builder
async fn stream_trades<LiquidExchange, IlliquidExchange>(
    market: &Instrument,
) -> UnboundedReceiver<MarketEvent<DataKind>>
where
    LiquidExchange: ExecutionClient,
    IlliquidExchange: ExecutionClient,
{
    let liquid_id = LiquidExchange::CLIENT;
    let illiquid_id = IlliquidExchange::CLIENT;

    let streams = dynamic::DynamicStreams::init([vec![
        (
            liquid_id,
            market.base.clone(),
            market.quote.clone(),
            StreamKind::L2,
        ),
        (
            illiquid_id,
            market.base.clone(),
            market.quote.clone(),
            StreamKind::L2,
        ),
        (
            liquid_id,
            market.base.clone(),
            market.quote.clone(),
            StreamKind::AggTrades,
        ),
        (
            illiquid_id,
            market.base.clone(),
            market.quote.clone(),
            StreamKind::Trades,
        ),
    ]])
    .await
    .unwrap();

    let mut data = streams.select_all::<MarketEvent<DataKind>>();

    let (tx, rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        while let Some(event) = data.next().await {
            // println!("{:?}", event);
            let _ = tx.send(event);
        }
    });

    rx
}
