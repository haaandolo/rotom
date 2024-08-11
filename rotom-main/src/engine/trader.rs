use std::collections::VecDeque;

use super::error::EngineError;
use crate::{
    data::{Feed, MarketGenerator},
    event::Event,
    execution::ExecutionClient,
    strategy::SignalGenerator,
};
use rotom_data::event_models::market_event::{DataKind, MarketEvent};

/*----- */
// Trader Lego
/*----- */
pub struct TraderLego<Data, Strategy, Execution>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
    Strategy: SignalGenerator,
    Execution: ExecutionClient,
{
    pub data: Data,
    pub stategy: Strategy,
    pub execution: Execution,
}

/*----- */
// Trader
/*----- */
pub struct Trader<Data, Strategy, Execution>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
    Strategy: SignalGenerator,
    Execution: ExecutionClient,
{
    pub data: Data,
    pub stategy: Strategy,
    pub execution: Execution,
    pub event_queue: VecDeque<Event>,
}

impl<Data, Strategy, Execution> Trader<Data, Strategy, Execution>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
    Strategy: SignalGenerator,
    Execution: ExecutionClient,
{
    pub fn new(lego: TraderLego<Data, Strategy, Execution>) -> Self {
        Self {
            data: lego.data,
            stategy: lego.stategy,
            execution: lego.execution,
            event_queue: VecDeque::with_capacity(4),
        }
    }

    pub fn builder() -> TraderBuilder<Data, Strategy, Execution> {
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
pub struct TraderBuilder<Data, Strategy, Execution>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
    Strategy: SignalGenerator,
    Execution: ExecutionClient,
{
    pub data: Option<Data>,
    pub strategy: Option<Strategy>,
    pub execution: Option<Execution>,
}

impl<Data, Strategy, Execution> TraderBuilder<Data, Strategy, Execution>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
    Strategy: SignalGenerator,
    Execution: ExecutionClient,
{
    pub fn new() -> Self {
        Self {
            data: None,
            strategy: None,
            execution: None,
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

    pub fn build(self) -> Result<Trader<Data, Strategy, Execution>, EngineError> {
        Ok(Trader {
            data: self.data.ok_or(EngineError::BuilderIncomplete("data"))?,
            stategy: self
                .strategy
                .ok_or(EngineError::BuilderIncomplete("strategy"))?,
            execution: self
                .execution
                .ok_or(EngineError::BuilderIncomplete("execution"))?,
            event_queue: VecDeque::with_capacity(2),
        })
    }
}
