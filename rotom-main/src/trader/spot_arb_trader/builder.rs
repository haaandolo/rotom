use std::collections::HashMap;

use futures::StreamExt;
use rotom_data::{
    model::market_event::{DataKind, MarketEvent},
    shared::subscription_models::{ExchangeId, Instrument, StreamKind},
    streams::builder::dynamic,
    MarketFeed,
};
use rotom_oms::{
    exchange::{ExecutionClient, ExecutionResponseChannel},
    execution_manager::builder::TraderId,
    model::order::ExecutionRequest,
};
use tokio::sync::mpsc::{self, Sender, UnboundedReceiver};
use uuid::Uuid;

use crate::engine::Command;

use super::{
    generate_decision::SpotArbOrderGenerator,
    trader::{SpotArbTrader, SpotArbTraderExecutionState, SpotArbTraderMetaData},
};

#[derive(Debug)]
pub struct SpotArbTradersBuilder<'a> {
    // Vec of traders to go into the engine
    pub traders: Vec<SpotArbTrader>,
    // Temp reference to HashMap of ExecutionManager ExecutionRequest tx's, for traders to clone
    pub execution_request_txs: &'a HashMap<ExchangeId, mpsc::UnboundedSender<ExecutionRequest>>,
    // Engine can send remote command's to the trader
    pub engine_command_tx: HashMap<TraderId, Sender<Command>>,
}

impl<'a> SpotArbTradersBuilder<'a> {
    pub fn new(
        execution_request_txs: &'a HashMap<ExchangeId, mpsc::UnboundedSender<ExecutionRequest>>,
    ) -> Self {
        Self {
            traders: Vec::new(),
            execution_request_txs,
            engine_command_tx: HashMap::new(),
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

            // Insert trader tx to receive remote commands
            self.engine_command_tx.insert(trader_id, engine_command_tx);

            // Build the trader
            let arb_trader = SpotArbTrader::builder()
                .engine_id(Uuid::new_v4())
                .trader_id(trader_id)
                .command_rx(engine_command_rx)
                .data(MarketFeed::new(stream_trades::<LiquidExchange, IlliquidExchange>(&market).await))
                .liquid_exchange_execution(
                    self.execution_request_txs
                        .get(&LiquidExchange::CLIENT)
                        .unwrap_or_else(|| {
                            panic!(
                                "Failed get liquid execution request tx for spot arb trader: {:#?}",
                                LiquidExchange::CLIENT
                            )
                        })
                        .clone(),
                )
                .illiquid_exchange_execution(
                    self.execution_request_txs
                        .get(&IlliquidExchange::CLIENT)
                        .unwrap_or_else(|| {
                            panic!(
                                "Failed get illiquid execution request tx for spot arb trader: {:#?}",
                                IlliquidExchange::CLIENT
                            )
                        })
                        .clone(),
                )
                .execution_response_channel(ExecutionResponseChannel::default())
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

    pub fn build(self) -> (Vec<SpotArbTrader>, HashMap<TraderId, Sender<Command>>) {
        (self.traders, self.engine_command_tx)
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
            println!("{:?}", event);
            let _ = tx.send(event);
        }
    });

    rx
}
