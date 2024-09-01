use std::{collections::VecDeque, sync::Arc};

use parking_lot::Mutex;
use rotom_data::{event_models::market_event::{DataKind, MarketEvent}, Feed, Market, MarketGenerator};
use rotom_strategy::{SignalForceExit, SignalGenerator};
use tokio::sync::mpsc;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::{
    engine::{error::EngineError, Command},
    event::{Event, EventTx, MessageTransmitter},
    execution::ExecutionClient,
    oms::{FillUpdater, MarketUpdater, OrderGenerator},
};

use super::TraderRun;

/*----- */
// Single Market Trader Lego
/*----- */
#[derive(Debug)]
pub struct SingleMarketTraderLego<Data, Strategy, Execution, Portfolio>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
    Strategy: SignalGenerator,
    Execution: ExecutionClient,
    Portfolio: MarketUpdater + OrderGenerator + FillUpdater,
{
    pub engine_id: Uuid,
    pub command_rx: mpsc::Receiver<Command>,
    pub event_tx: EventTx,
    pub markets: Market,
    pub data: Data,
    pub stategy: Strategy,
    pub execution: Execution,
    pub porfolio: Arc<Mutex<Portfolio>>,
}

/*----- */
// Single Market Trader
/*----- */
#[derive(Debug)]
pub struct SingleMarketTrader<Data, Strategy, Execution, Portfolio>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
    Strategy: SignalGenerator,
    Execution: ExecutionClient,
    Portfolio: MarketUpdater + OrderGenerator + FillUpdater,
{
    engine_id: Uuid,
    command_rx: mpsc::Receiver<Command>,
    event_tx: EventTx,
    markets: Market,
    data: Data,
    stategy: Strategy,
    execution: Execution,
    event_queue: VecDeque<Event>,
    portfolio: Arc<Mutex<Portfolio>>,
}

impl<Data, Strategy, Execution, Portfolio> SingleMarketTrader<Data, Strategy, Execution, Portfolio>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
    Strategy: SignalGenerator,
    Execution: ExecutionClient,
    Portfolio: MarketUpdater + OrderGenerator + FillUpdater,
{
    pub fn new(lego: SingleMarketTraderLego<Data, Strategy, Execution, Portfolio>) -> Self {
        Self {
            engine_id: lego.engine_id,
            command_rx: lego.command_rx,
            event_tx: lego.event_tx,
            markets: lego.markets,
            data: lego.data,
            stategy: lego.stategy,
            execution: lego.execution,
            event_queue: VecDeque::with_capacity(4),
            portfolio: lego.porfolio,
        }
    }

    pub fn builder() -> SingleMarketTraderBuilder<Data, Strategy, Execution, Portfolio> {
        SingleMarketTraderBuilder::new()
    }
}

/*----- */
// Impl Trader trait for Single Market Trader
/*----- */
impl<Data, Strategy, Execution, Portfolio> TraderRun
    for SingleMarketTrader<Data, Strategy, Execution, Portfolio>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
    Strategy: SignalGenerator,
    Execution: ExecutionClient,
    Portfolio: MarketUpdater + OrderGenerator + FillUpdater,
{
    fn receive_remote_command(&mut self) -> Option<Command> {
        match self.command_rx.try_recv() {
            Ok(command) => {
                debug!(
                    engine_id = &*self.engine_id.to_string(),
                    markets = &*format!("{:?}", self.markets),
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

    fn run(mut self) {
        'trading: loop {
            // Check for mew remote Commands
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

            // If the Feed<MarketEvent> yields, populate the event queue with the next MarketEvent
            match self.data.next() {
                Feed::Next(market) => {
                    // self.event_tx.send(Event::Market(market.clone()));
                    self.event_queue.push_back(Event::Market(market));
                }
                Feed::UnHealthy => {
                    warn!(
                        engine_id = %self.engine_id,
                        market = ?self.markets,
                        action = "continuing while waiting for healthy Feed",
                        "MarketFeed unhealthy"
                    );
                    continue 'trading;
                }
                Feed::Finished => break 'trading,
            }

            // This while loop handles Events from the event_queue it will break if the event_queue
            // empty and requires another MarketEvent
            while let Some(event) = self.event_queue.pop_front() {
                match event {
                    Event::Market(market_event) => {
                        if let Some(signal) = self.stategy.generate_signal(&market_event) {
                            self.event_tx.send(Event::Signal(signal.clone()));
                            self.event_queue.push_back(Event::Signal(signal))
                        }

                        if let Some(position_update) = self
                            .portfolio
                            .lock()
                            .update_from_market(&market_event)
                            .expect("Failed to update Portfolio from MarketEvent")
                        {
                            self.event_tx.send(Event::PositionUpdate(position_update));
                        }
                    }
                    Event::Signal(signal) => {
                        if let Some(order) = self
                            .portfolio
                            .lock()
                            .generate_order(&signal)
                            .expect("Failed to generate order")
                        {
                            self.event_tx.send(Event::OrderNew(order.clone()));
                            self.event_queue.push_back(Event::OrderNew(order));
                        }
                    }
                    Event::SignalForceExit(signal_force_exit) => {
                        if let Some(order) = self
                            .portfolio
                            .lock()
                            .generate_exit_order(signal_force_exit)
                            .expect("Failed to generate forced exit order")
                        {
                            self.event_tx.send(Event::OrderNew(order.clone()));
                            self.event_queue.push_back(Event::OrderNew(order));
                        }
                    }
                    Event::OrderNew(order) => {
                        let fill = self
                            .execution
                            .generate_fill(&order)
                            .expect("Failed to generate");

                        self.event_tx.send(Event::Fill(fill.clone()));
                        self.event_queue.push_back(Event::Fill(fill));
                    }
                    Event::Fill(fill) => {
                        let fill_side_effect_events = self
                            .portfolio
                            .lock()
                            .update_from_fill(&fill)
                            .expect("Failed to update Portfolio from fill");
                        self.event_tx.send_many(fill_side_effect_events);
                    }
                    _ => {}
                }
            }

            debug!(
                engine_id = &*self.engine_id.to_string(),
                market = &*format!("{:?}", self.markets),
                message = "Trader trading loop stopped"
            )
        }
    }
}

/*----- */
// Single Market Trader builder
/*----- */
#[derive(Debug, Default)]
pub struct SingleMarketTraderBuilder<Data, Strategy, Execution, Portfolio>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
    Strategy: SignalGenerator,
    Execution: ExecutionClient,
    Portfolio: MarketUpdater + OrderGenerator + FillUpdater,
{
    pub engine_id: Option<Uuid>,
    pub command_rx: Option<mpsc::Receiver<Command>>,
    pub event_tx: Option<EventTx>,
    pub markets: Option<Market>,
    pub data: Option<Data>,
    pub strategy: Option<Strategy>,
    pub execution: Option<Execution>,
    pub portfolio: Option<Arc<Mutex<Portfolio>>>,
}

impl<Data, Strategy, Execution, Portfolio>
    SingleMarketTraderBuilder<Data, Strategy, Execution, Portfolio>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
    Strategy: SignalGenerator,
    Execution: ExecutionClient,
    Portfolio: MarketUpdater + OrderGenerator + FillUpdater,
{
    pub fn new() -> Self {
        Self {
            engine_id: None,
            command_rx: None,
            event_tx: None,
            markets: None,
            data: None,
            strategy: None,
            execution: None,
            portfolio: None,
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

    pub fn event_tx(self, value: EventTx) -> Self {
        Self {
            event_tx: Some(value),
            ..self
        }
    }

    pub fn market(self, value: Market) -> Self {
        Self {
            markets: Some(value),
            ..self
        }
    }

    pub fn data(self, value: Data) -> Self {
        Self {
            data: Some(value),
            ..self
        }
    }

    pub fn strategy(self, value: Strategy) -> Self {
        Self {
            strategy: Some(value),
            ..self
        }
    }

    pub fn execution(self, value: Execution) -> Self {
        Self {
            execution: Some(value),
            ..self
        }
    }

    pub fn portfolio(self, value: Arc<Mutex<Portfolio>>) -> Self {
        Self {
            portfolio: Some(value),
            ..self
        }
    }

    pub fn build(
        self,
    ) -> Result<SingleMarketTrader<Data, Strategy, Execution, Portfolio>, EngineError> {
        Ok(SingleMarketTrader {
            engine_id: self
                .engine_id
                .ok_or(EngineError::BuilderIncomplete("engine_id"))?,
            command_rx: self
                .command_rx
                .ok_or(EngineError::BuilderIncomplete("command_rx"))?,
            event_tx: self
                .event_tx
                .ok_or(EngineError::BuilderIncomplete("event_tx"))?,
            markets: self
                .markets
                .ok_or(EngineError::BuilderIncomplete("markets"))?,
            data: self.data.ok_or(EngineError::BuilderIncomplete("data"))?,
            stategy: self
                .strategy
                .ok_or(EngineError::BuilderIncomplete("strategy"))?,
            execution: self
                .execution
                .ok_or(EngineError::BuilderIncomplete("execution"))?,
            event_queue: VecDeque::with_capacity(2),
            portfolio: self
                .portfolio
                .ok_or(EngineError::BuilderIncomplete("portfolio"))?,
        })
    }
}
