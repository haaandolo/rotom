use std::{collections::VecDeque, sync::Arc};

use super::error::EngineError;
use crate::{
    data::{Feed, MarketGenerator}, event::Event, execution::ExecutionClient, oms::{FillUpdater, MarketUpdater, OrderGenerator}, strategy::SignalGenerator
};
use futures::lock::Mutex;
use rotom_data::event_models::market_event::{DataKind, MarketEvent};

/*----- */
// Trader Lego
/*----- */
pub struct TraderLego<Data, Strategy, Execution, Portfolio>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
    Strategy: SignalGenerator,
    Execution: ExecutionClient,
    Portfolio: MarketUpdater + OrderGenerator + FillUpdater
{
    pub data: Data,
    pub stategy: Strategy,
    pub execution: Execution,
    pub porfolio: Arc<Mutex<Portfolio>>,
}

/*----- */
// Trader
/*----- */
pub struct Trader<Data, Strategy, Execution, Portfolio>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
    Strategy: SignalGenerator,
    Execution: ExecutionClient,
    Portfolio: MarketUpdater + OrderGenerator + FillUpdater
{
    pub data: Data,
    pub stategy: Strategy,
    pub execution: Execution,
    pub event_queue: VecDeque<Event>,
    pub portfolio: Arc<Mutex<Portfolio>>
}

impl<Data, Strategy, Execution, Portfolio> Trader<Data, Strategy, Execution, Portfolio>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
    Strategy: SignalGenerator,
    Execution: ExecutionClient,
    Portfolio: MarketUpdater + OrderGenerator + FillUpdater
{
    pub fn new(lego: TraderLego<Data, Strategy, Execution, Portfolio>) -> Self {
        Self {
            data: lego.data,
            stategy: lego.stategy,
            execution: lego.execution,
            event_queue: VecDeque::with_capacity(4),
            portfolio: lego.porfolio
        }
    }

    pub fn builder() -> TraderBuilder<Data, Strategy, Execution, Portfolio> {
        TraderBuilder::new()
    }

    pub fn run(mut self) {
        loop {
            // If the Feed<MarketEvent> yields, populate the event queue with the next MarketEvent
            match self.data.next() {
                Feed::Next(market_event) => self.event_queue.push_back(Event::Market(market_event)),
                Feed::UnHealthy => {
                    continue; // todo: log error
                }
                Feed::Finished => {
                    break; // todo: log finshed?
                }
            }

            // This while loop handles Events from the event_queue it will break if the event_queue
            // empty and requires another MarketEvent
            while let Some(event) = self.event_queue.pop_front() {
                match event {
                    Event::Market(market_event) => {
                        if let Some(signal) = self.stategy.generate_signal(&market_event) {
                            self.event_queue.push_back(Event::Signal(signal))
                        }
                    }
                    Event::Signal(_signal) => {
                        // self.event_queue.push_back(Event::OrderNew(signal))
                    }
                    _ => {}
                }
            }
        }
    }
}

/*----- */
// Trader builder
/*----- */
#[derive(Debug, Default)]
pub struct TraderBuilder<Data, Strategy, Execution, Portfolio>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
    Strategy: SignalGenerator,
    Execution: ExecutionClient,
    Portfolio: MarketUpdater + OrderGenerator + FillUpdater
{
    pub data: Option<Data>,
    pub strategy: Option<Strategy>,
    pub execution: Option<Execution>,
    pub portfolio: Option<Arc<Mutex<Portfolio>>>
}

impl<Data, Strategy, Execution, Portfolio> TraderBuilder<Data, Strategy, Execution, Portfolio>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
    Strategy: SignalGenerator,
    Execution: ExecutionClient,
    Portfolio: MarketUpdater + OrderGenerator + FillUpdater
{
    pub fn new() -> Self {
        Self {
            data: None,
            strategy: None,
            execution: None,
            portfolio: None
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

    pub fn build(self) -> Result<Trader<Data, Strategy, Execution, Portfolio>, EngineError> {
        Ok(Trader {
            data: self.data.ok_or(EngineError::BuilderIncomplete("data"))?,
            stategy: self
                .strategy
                .ok_or(EngineError::BuilderIncomplete("strategy"))?,
            execution: self
                .execution
                .ok_or(EngineError::BuilderIncomplete("execution"))?,
            event_queue: VecDeque::with_capacity(2),
            portfolio: self.portfolio.ok_or(EngineError::BuilderIncomplete("portfolio"))?
        })
    }
}
