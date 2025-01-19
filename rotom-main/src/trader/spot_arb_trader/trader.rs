use chrono::Utc;
use rotom_data::shared::subscription_models::Instrument;
use rotom_data::MarketFeed;
use rotom_data::{
    model::market_event::{DataKind, MarketEvent},
    shared::subscription_models::ExchangeId,
    Feed, MarketGenerator,
};
use rotom_oms::execution_manager::builder::TraderId;
use rotom_oms::model::execution_request::{ExecutionRequest, OpenOrder};
use rotom_oms::model::execution_response::ExecutionResponse;
use rotom_oms::model::{ClientOrderId, Order};
use rotom_oms::{event::Event, model::order::OrderEvent};
use rotom_strategy::SignalForceExit;
use std::collections::VecDeque;
use tokio::sync::mpsc;
use tracing::{debug, warn};
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
    execution_request_tx: mpsc::UnboundedSender<Order<ExecutionRequest>>,
    execution_response_rx: mpsc::UnboundedReceiver<ExecutionResponse>,
    order_generator: SpotArbOrderGenerator,
    event_queue: VecDeque<Event>,
    meta_data: SpotArbTraderMetaData,
}

impl SpotArbTrader {
    pub fn builder() -> SpotArbTraderBuilder {
        SpotArbTraderBuilder::new()
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

    fn subscribe_to_execution_manager(&mut self) {} // todo; rm

    fn run(mut self) {
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
                                client_order_id: ClientOrderId::random(),
                                price: 1.50,
                                quantity: 5.0,
                                notional_amount: 1.50 * 5.0,
                                decision: rotom_strategy::Decision::Long,
                                order_kind: rotom_oms::model::OrderKind::Limit,
                                instrument: self.meta_data.market.clone(),
                            };

                            let order = Order {
                                trader_id: self.trader_id,
                                exchange: self.meta_data.liquid_exchange,
                                requested_time: Utc::now(),
                                request_response: ExecutionRequest::Open(order_request),
                            };

                            let test = self.execution_request_tx.send(order);

                            self.meta_data.order = Some(new_order);
                        }
                    },
                    _ => {}
                }
            }

            /*----- 4. Process any execution updates ----- */
            match self.execution_response_rx.try_recv() {
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
    pub execution_request_tx: Option<mpsc::UnboundedSender<Order<ExecutionRequest>>>,
    pub execution_response_rx: Option<mpsc::UnboundedReceiver<ExecutionResponse>>,
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
            execution_request_tx: None,
            order_generator: None,
            execution_response_rx: None,
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

    pub fn execution_request_tx(
        self,
        value: mpsc::UnboundedSender<Order<ExecutionRequest>>,
    ) -> Self {
        Self {
            execution_request_tx: Some(value),
            ..self
        }
    }

    pub fn execution_response_rx(self, value: mpsc::UnboundedReceiver<ExecutionResponse>) -> Self {
        Self {
            execution_response_rx: Some(value),
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
            execution_request_tx: self
                .execution_request_tx
                .ok_or(EngineError::BuilderIncomplete("execution_request"))?,
            execution_response_rx: self
                .execution_response_rx
                .ok_or(EngineError::BuilderIncomplete("execution_response_rx"))?,
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
