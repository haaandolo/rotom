use std::collections::HashMap;

use futures::StreamExt;
use rotom_data::{
    model::market_event::{DataKind, MarketEvent},
    shared::subscription_models::{ExchangeId, Instrument, StreamKind},
    streams::builder::dynamic,
    MarketFeed,
};
use rotom_oms::{
    exchange::{ExecutionClient, ExecutionRequestChannel},
    execution_manager::builder::TraderId,
    model::{account_data::AccountData, order::ExecutionRequest},
};
use tokio::sync::mpsc::{self, Sender, UnboundedReceiver, UnboundedSender};
use uuid::Uuid;

use crate::engine::Command;

use super::{
    generate_decision::SpotArbOrderGenerator,
    trader::{SpotArbTrader, SpotArbTraderExecutionState, SpotArbTraderMetaData},
};

#[derive(Debug, Default)]
pub struct SpotArbTradersBuilder {
    pub traders: Vec<SpotArbTrader>,
    pub execution_request_tx: HashMap<ExchangeId, UnboundedSender<ExecutionRequest>>,
    pub execution_request_rx: HashMap<ExchangeId, UnboundedReceiver<ExecutionRequest>>,
    pub execution_response_tx: HashMap<ExchangeId, HashMap<TraderId, Sender<AccountData>>>,
    pub engine_command_tx: HashMap<TraderId, Sender<Command>>
}

impl SpotArbTradersBuilder {
    pub async fn add_traders<LiquidExchange, IlliquidExchange>(
        mut self,
        markets: Vec<Instrument>,
    ) -> Self
    where
        LiquidExchange: ExecutionClient<
            ExecutionRequestChannel = ExecutionRequestChannel,
        >,
        IlliquidExchange: ExecutionClient<
            ExecutionRequestChannel = ExecutionRequestChannel,
        >,
    {
        // Init liquid exchange execution request channel and add to hashmap
        let liquid_execution_request_channel = LiquidExchange::ExecutionRequestChannel::default();

        self.execution_request_tx
            .entry(LiquidExchange::CLIENT)
            .or_insert(liquid_execution_request_channel.tx);

        self.execution_request_rx
            .entry(LiquidExchange::CLIENT)
            .or_insert(liquid_execution_request_channel.rx);

        // Init illiquid exchange execution request channel and add to hashmap
        let illiquid_execution_request_channel =
            IlliquidExchange::ExecutionRequestChannel::default();

        self.execution_request_tx
            .entry(IlliquidExchange::CLIENT)
            .or_insert(illiquid_execution_request_channel.tx);

        self.execution_request_rx
            .entry(IlliquidExchange::CLIENT)
            .or_insert(illiquid_execution_request_channel.rx);

        // Add spot arb traders
        for market in markets.into_iter() {
            // Initalise trader id and channels
            let trader_id = TraderId(Uuid::new_v4());
            let (trader_command_tx, trader_command_rx) = mpsc::channel(10);
            let (execution_response_tx, execution_response_rx) = mpsc::channel(10);

            // Insert trader tx to receive remote commands
            self.engine_command_tx.insert(trader_id.clone(), trader_command_tx);

            // Insert tx to send updates back to this trader from liquid exchange
            self.execution_response_tx.entry(LiquidExchange::CLIENT)
                .or_default()
                .insert(trader_id.clone(), execution_response_tx.clone());

            // Insert tx to send updates back to this trader from illiquid exchange
            self.execution_response_tx.entry(IlliquidExchange::CLIENT)
                .or_default()
                .insert(trader_id.clone(), execution_response_tx.clone());

            // Build the trader
            let arb_trader = SpotArbTrader::builder()
                .engine_id(Uuid::new_v4())
                .trader_id(trader_id.clone())
                .command_rx(trader_command_rx)
                .data(MarketFeed::new(stream_trades().await))
                .liquid_exchange(
                    self.execution_request_tx
                        .get(&LiquidExchange::CLIENT)
                        .unwrap_or_else(|| {
                            panic!(
                                "Failed get liquid execution request tx for spot arb trader: {:#?}",
                                LiquidExchange::CLIENT
                            )
                        })
                        .clone(),
                )
                .illiquid_exchange(
                    self.execution_request_tx
                        .get(&IlliquidExchange::CLIENT)
                        .unwrap_or_else(|| {
                            panic!(
                                "Failed get illiquid execution request tx for spot arb trader: {:#?}",
                                IlliquidExchange::CLIENT
                            )
                        })
                        .clone(),
                )
                .order_generator(SpotArbOrderGenerator::default())
                .order_update_rx(execution_response_rx)
                .meta_data(SpotArbTraderMetaData { 
                    order: None, 
                    execution_state: SpotArbTraderExecutionState::NoPosition, 
                    liquid_exchange: LiquidExchange::CLIENT, 
                    illiquid_exchange: IlliquidExchange::CLIENT, 
                    market }
                )
                .build()
                .unwrap();

            self.traders.push(arb_trader);
        }

        self
    }
}

// todo: make this into a spot arb specfic stream builder
async fn stream_trades() -> UnboundedReceiver<MarketEvent<DataKind>> {
    let streams = dynamic::DynamicStreams::init([vec![
        (ExchangeId::BinanceSpot, "op", "usdt", StreamKind::L2),
        (ExchangeId::PoloniexSpot, "op", "usdt", StreamKind::L2),
        (ExchangeId::BinanceSpot, "op", "usdt", StreamKind::AggTrades),
        (ExchangeId::PoloniexSpot, "op", "usdt", StreamKind::Trades),
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
