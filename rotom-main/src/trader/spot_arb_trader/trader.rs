use rotom_data::shared::subscription_models::Instrument;
use rotom_data::streams::builder::single::ExchangeChannel;
use rotom_data::MarketFeed;
use rotom_data::{
    model::market_event::{DataKind, MarketEvent},
    shared::subscription_models::ExchangeId,
    Feed, MarketGenerator,
};
use rotom_oms::execution_manager::builder::TraderId;
use rotom_oms::model::account_data::ExecutionResponse;
use rotom_oms::model::order::{ExecutionManagerSubscribe, OpenOrder};
use rotom_oms::model::ClientOrderId;
use rotom_oms::{
    event::Event,
    model::order::{ExecutionRequest, OrderEvent},
};
use rotom_strategy::SignalForceExit;
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::{debug, error, warn};
use uuid::Uuid;

use crate::engine::{error::EngineError, Command};
use crate::trader::TraderRun;

use super::generate_decision::SpotArbOrderGenerator;

/*----- */
// Spot Arb Trader - Metadata info
/*----- */
#[derive(Debug, PartialEq, Clone)]
pub enum SpotArbTraderExecutionState {
    NoPosition,
    TakerBuyLiquid,
    TakerSellLiquid,
    TransferToLiquid,
    MakerBuyIlliquid,
    MakerSellIlliquid,
    TransferToIlliquid,
}

#[derive(Debug)]
pub struct SpotArbTraderMetaData {
    pub order: Option<OrderEvent>,
    pub execution_state: SpotArbTraderExecutionState,
    pub liquid_exchange: ExchangeId,
    pub illiquid_exchange: ExchangeId,
    pub market: Instrument,
}

/*----- */
// Spot Arb Trader
/*----- */
#[derive(Debug)]
pub struct SpotArbTrader {
    engine_id: Uuid,
    trader_id: TraderId,
    command_rx: mpsc::Receiver<Command>,
    data: MarketFeed<MarketEvent<DataKind>>,
    liquid_exchange_execution: mpsc::UnboundedSender<ExecutionRequest>,
    illiquid_exchange_execution: mpsc::UnboundedSender<ExecutionRequest>,
    execution_response_channel: ExchangeChannel<ExecutionResponse>,
    order_generator: SpotArbOrderGenerator,
    event_queue: VecDeque<Event>,
    meta_data: SpotArbTraderMetaData,
}

impl SpotArbTrader {
    pub fn builder() -> SpotArbTraderBuilder {
        SpotArbTraderBuilder::new()
    }

    pub fn send_liquid_execution_request(&self, request: ExecutionRequest) {
        if let Err(error) = self.liquid_exchange_execution.send(request) {
            debug!(
                message = "Failed to send execution request for liquid exchange in SpotArbTrader",
                trader_meta_data = ?self.meta_data,
                error = %error
            )
        }
    }

    pub fn send_illiquid_execution_request(&self, request: ExecutionRequest) {
        if let Err(error) = self.illiquid_exchange_execution.send(request) {
            debug!(
                message = "Failed to send execution request for illiquid exchange in SpotArbTrader",
                trader_meta_data = ?self.meta_data,
                error = %error
            )
        }
    }
}

/*----- */
// Impl Trader trait for Single Market Trader
/*----- */
impl TraderRun for SpotArbTrader {
    fn receive_remote_command(&mut self) -> Option<Command> {
        match self.command_rx.try_recv() {
            Ok(command) => {
                debug!(
                    engine_id = &*self.engine_id.to_string(),
                    trader_id = %self.trader_id,
                    spot_arb_meta_data = &*format!("{:?}", self.meta_data),
                    command = &*format!("{:?}", command),
                    message = "Trader received remote command"
                );
                Some(command)
            }
            Err(error) => match error {
                mpsc::error::TryRecvError::Empty => None,
                mpsc::error::TryRecvError::Disconnected => {
                    warn!(
                        action = "Sythesising a Command::Terminate",
                        message = "Remote Command transmitter has been dropped"
                    );
                    Some(Command::Terminate(
                        "Remote command transmitter dropped".to_owned(),
                    ))
                }
            },
        }
    }

    fn subscribe_to_execution_manager(&mut self) {
        // Create subscription request
        let subscription_request = ExecutionManagerSubscribe {
            trader_id: self.trader_id,
            instruments: vec![self.meta_data.market.clone()],
            execution_response_tx: self.execution_response_channel.tx.clone(),
        };

        // Subscribe to ExecutionManagers
        self.send_liquid_execution_request(ExecutionRequest::Subscribe(
            subscription_request.clone(),
        ));

        self.send_illiquid_execution_request(ExecutionRequest::Subscribe(
            subscription_request.clone(),
        ));

        // Make sure to receive 2 successful responses back from ExecutionManger before continuing
        let start = Instant::now();
        let timeout = Duration::from_secs(10);
        let mut success_msg_received = Vec::with_capacity(2);
        loop {
            match self.execution_response_channel.rx.try_recv() {
                Ok(response) => {
                    if let ExecutionResponse::Subscribed(exchange_id) = response {
                        success_msg_received.push(exchange_id);
                    };

                    if success_msg_received.len() == 2 {
                        break;
                    }
                }
                Err(error) => {
                    if start.elapsed() >= timeout {
                        error!(
                                "Failed to subscribe to ExecutionManager's for SpotArbTrader. Error: {:#?}. Trader meta data: {:#?}. Successful subscription message received from: {:#?}",
                                error,
                                self.meta_data,
                                success_msg_received
                            );
                        std::process::exit(1);
                    }
                }
            }
        }
    }

    fn run(mut self) {
        // Subscribe to ExecutionManagers, panic if unsuccessful
        self.subscribe_to_execution_manager();

        // If ExecutionManger subscription is successful then go to trading loop
        'trading: loop {
            /*----- 1. Check for remote cmd ----- */
            while let Some(command) = self.receive_remote_command() {
                match command {
                    Command::Terminate(_) => break 'trading,
                    Command::ExitPosition(market) => {
                        self.event_queue
                            .push_back(Event::SignalForceExit(SignalForceExit::from(market)));
                    }
                    _ => continue,
                }
            }

            /*----- 2. Get Market Event Update ----- */
            match self.data.next() {
                Feed::Next(market) => {
                    self.event_queue.push_back(Event::Market(market));
                }
                Feed::UnHealthy => {
                    warn!(
                        engine_id = %self.engine_id,
                        trader_id = %self.trader_id,
                        spot_arb_meta_data = &*format!("{:?}", self.meta_data),
                        action = "continuing while waiting for healthy Feed",
                        "MarketFeed unhealthy"
                    );
                    continue 'trading;
                }
                Feed::Finished => break 'trading,
            }

            /*----- 3. Process Event Loop ----- */
            while let Some(event) = self.event_queue.pop_front() {
                match event {
                    Event::Market(market_event) => {
                        // println!("#################");
                        // println!("Market data");
                        // println!("#################");
                        // println!("{:?}", market_event);

                        // Process latest market event and generate order if applicable
                        if let Some(new_order) =
                            self.order_generator.process_market_event(&market_event)
                        {
                            self.event_queue.push_back(Event::OrderNew(new_order));
                        }
                    }
                    Event::OrderNew(new_order) => match &self.meta_data.order {
                        Some(_) => {}
                        None => {
                            println!("blocking");
                            std::thread::sleep(std::time::Duration::from_secs(1));
                            println!("unblocked");

                            let order_request = OpenOrder {
                                trader_id: self.trader_id,
                                client_order_id: ClientOrderId::random(),
                                price: 1.50,
                                quantity: 50.0,
                                notional_amount: 1.50 * 5.0,
                                decision: rotom_strategy::Decision::Long,
                                order_kind: rotom_oms::model::OrderKind::Limit,
                                instrument: self.meta_data.market.clone(),
                            };

                            self.send_liquid_execution_request(ExecutionRequest::Open(
                                order_request,
                            ));

                            self.meta_data.order = Some(new_order);
                        }
                    },
                    _ => {}
                }
            }

            /*----- 4. Process any execution updates ----- */
            match self.execution_response_channel.rx.try_recv() {
                Ok(execution_result) => {}
                Err(error) => {}
            }
        }

        debug!(
            engine_id = &*self.engine_id.to_string(),
            trader_id = %self.trader_id,
            spot_arb_meta_data = &*format!("{:?}", self.meta_data),
            message = "Trader trading loop stopped"
        )
    }
}

/*----- */
// SpotArbTraderBuilder
/*----- */
#[derive(Debug, Default)]
pub struct SpotArbTraderBuilder {
    pub engine_id: Option<Uuid>,
    pub trader_id: Option<TraderId>,
    pub command_rx: Option<mpsc::Receiver<Command>>,
    pub data: Option<MarketFeed<MarketEvent<DataKind>>>,
    pub liquid_exchange_execution: Option<mpsc::UnboundedSender<ExecutionRequest>>,
    pub illiquid_exchange_execution: Option<mpsc::UnboundedSender<ExecutionRequest>>,
    pub execution_response_channel: Option<ExchangeChannel<ExecutionResponse>>,
    pub order_generator: Option<SpotArbOrderGenerator>,
    pub meta_data: Option<SpotArbTraderMetaData>,
}

impl SpotArbTraderBuilder {
    pub fn new() -> Self {
        Self {
            engine_id: None,
            trader_id: None,
            command_rx: None,
            data: None,
            liquid_exchange_execution: None,
            illiquid_exchange_execution: None,
            order_generator: None,
            execution_response_channel: None,
            meta_data: None,
        }
    }

    pub fn engine_id(self, value: Uuid) -> Self {
        Self {
            engine_id: Some(value),
            ..self
        }
    }

    pub fn trader_id(self, value: TraderId) -> Self {
        Self {
            trader_id: Some(value),
            ..self
        }
    }

    pub fn command_rx(self, value: mpsc::Receiver<Command>) -> Self {
        Self {
            command_rx: Some(value),
            ..self
        }
    }

    pub fn data(self, value: MarketFeed<MarketEvent<DataKind>>) -> Self {
        Self {
            data: Some(value),
            ..self
        }
    }

    pub fn liquid_exchange_execution(self, value: mpsc::UnboundedSender<ExecutionRequest>) -> Self {
        Self {
            liquid_exchange_execution: Some(value),
            ..self
        }
    }

    pub fn illiquid_exchange_execution(
        self,
        value: mpsc::UnboundedSender<ExecutionRequest>,
    ) -> Self {
        Self {
            illiquid_exchange_execution: Some(value),
            ..self
        }
    }

    pub fn execution_response_channel(self, value: ExchangeChannel<ExecutionResponse>) -> Self {
        Self {
            execution_response_channel: Some(value),
            ..self
        }
    }

    pub fn order_generator(self, value: SpotArbOrderGenerator) -> Self {
        Self {
            order_generator: Some(value),
            ..self
        }
    }

    pub fn meta_data(self, value: SpotArbTraderMetaData) -> Self {
        Self {
            meta_data: Some(value),
            ..self
        }
    }

    pub fn build(self) -> Result<SpotArbTrader, EngineError> {
        Ok(SpotArbTrader {
            engine_id: self
                .engine_id
                .ok_or(EngineError::BuilderIncomplete("engine_id"))?,
            trader_id: self
                .trader_id
                .ok_or(EngineError::BuilderIncomplete("trader_id"))?,
            command_rx: self
                .command_rx
                .ok_or(EngineError::BuilderIncomplete("command_rx"))?,
            data: self.data.ok_or(EngineError::BuilderIncomplete("data"))?,
            liquid_exchange_execution: self
                .liquid_exchange_execution
                .ok_or(EngineError::BuilderIncomplete("liquid_exchange_execution"))?,
            illiquid_exchange_execution: self.illiquid_exchange_execution.ok_or(
                EngineError::BuilderIncomplete("illiquid_exchange_execution"),
            )?,
            execution_response_channel: self
                .execution_response_channel
                .ok_or(EngineError::BuilderIncomplete("execution_response_channel"))?,
            order_generator: self
                .order_generator
                .ok_or(EngineError::BuilderIncomplete("order_generator"))?,
            event_queue: VecDeque::with_capacity(2),
            meta_data: self
                .meta_data
                .ok_or(EngineError::BuilderIncomplete("meta_data"))?,
        })
    }
}
