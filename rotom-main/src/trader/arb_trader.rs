use std::{collections::VecDeque, sync::Arc};

use parking_lot::Mutex;
use rotom_data::{
    event_models::market_event::{DataKind, MarketEvent},
    Feed, Market, MarketGenerator,
};
use rotom_oms::{
    event::{Event, EventTx, MessageTransmitter},
    execution::ExecutionClient,
    portfolio::portfolio_type::{FillUpdater, MarketUpdater, OrderGenerator},
};
use rotom_strategy::{SignalForceExit, SignalGenerator};
use tokio::sync::mpsc;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::engine::{error::EngineError, Command};

use super::TraderRun;

/*----- */
// Single Market Trader Lego
/*----- */
#[derive(Debug)]
pub struct ArbTraderLego<Data, Strategy, Execution, Portfolio>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
    Strategy: SignalGenerator,
    Execution: ExecutionClient,
    Portfolio: MarketUpdater + OrderGenerator + FillUpdater,
{
    pub engine_id: Uuid,
    pub command_rx: mpsc::Receiver<Command>,
    pub event_tx: EventTx,
    pub markets: Vec<Market>,
    pub data: Data,
    pub stategy: Strategy,
    pub execution: Execution,
    pub porfolio: Arc<Mutex<Portfolio>>,
}

/*----- */
// Single Market Trader
/*----- */
#[derive(Debug)]
pub struct ArbTrader<Data, Strategy, Execution, Portfolio>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
    Strategy: SignalGenerator,
    Execution: ExecutionClient,
    Portfolio: MarketUpdater + OrderGenerator,
{
    engine_id: Uuid,
    command_rx: mpsc::Receiver<Command>,
    event_tx: EventTx,
    markets: Vec<Market>,
    data: Data,
    stategy: Strategy,
    execution: Execution,
    event_queue: VecDeque<Event>,
    portfolio: Arc<Mutex<Portfolio>>,
}

impl<Data, Strategy, Execution, Portfolio> ArbTrader<Data, Strategy, Execution, Portfolio>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
    Strategy: SignalGenerator,
    Execution: ExecutionClient,
    Portfolio: MarketUpdater + OrderGenerator + FillUpdater,
{
    pub fn new(lego: ArbTraderLego<Data, Strategy, Execution, Portfolio>) -> Self {
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

    pub fn builder() -> ArbTraderBuilder<Data, Strategy, Execution, Portfolio> {
        ArbTraderBuilder::new()
    }
}

/*----- */
// Impl Trader trait for Single Market Trader
/*----- */
impl<Data, Strategy, Execution, Portfolio> TraderRun
    for ArbTrader<Data, Strategy, Execution, Portfolio>
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
                            // println!("##############################");
                            // println!("signal --> {:#?}", signal);
                            self.event_tx.send(Event::Signal(signal.clone()));
                            self.event_queue.push_back(Event::Signal(signal))
                        }

                        if let Some(position_update) = self
                            .portfolio
                            .lock()
                            .update_from_market(&market_event)
                            .expect("Failed to update Portfolio from MarketEvent")
                        {
                            // println!("##############################");
                            // println!("position update --> {:#?}", position_update);
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
                            println!("##############################");
                            println!("order --> {:#?}", order);
                            self.event_tx.send(Event::OrderNew(order.clone()));
                            self.event_queue.push_back(Event::OrderNew(order));
                        }
                    }
                    // Event::SignalForceExit(signal_force_exit) => {
                    //     if let Some(order) = self
                    //         .portfolio
                    //         .lock()
                    //         .generate_exit_order(signal_force_exit)
                    //         .expect("Failed to generate forced exit order")
                    //     {
                    //         self.event_tx.send(Event::OrderNew(order.clone()));
                    //         self.event_queue.push_back(Event::OrderNew(order));
                    //     }
                    // }
                    Event::OrderNew(order) => {
                        let fill = self
                            .execution
                            .generate_fill(&order)
                            .expect("Failed to generate");

                        println!("##############################");
                        println!("fill --> {:#?}", fill);
                        self.event_tx.send(Event::Fill(fill.clone()));
                        self.event_queue.push_back(Event::Fill(fill));
                    }
                    Event::Fill(fill) => {
                        let fill_side_effect_events = self
                            .portfolio
                            .lock()
                            .update_from_fill(&fill)
                            .expect("Failed to update Portfolio from fill");

                        println!("##############################");
                        println!("fill event --> {:#?}", fill_side_effect_events);
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
pub struct ArbTraderBuilder<Data, Strategy, Execution, Portfolio>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
    Strategy: SignalGenerator,
    Execution: ExecutionClient,
    Portfolio: MarketUpdater + OrderGenerator + FillUpdater,
{
    pub engine_id: Option<Uuid>,
    pub command_rx: Option<mpsc::Receiver<Command>>,
    pub event_tx: Option<EventTx>,
    pub markets: Option<Vec<Market>>,
    pub data: Option<Data>,
    pub strategy: Option<Strategy>,
    pub execution: Option<Execution>,
    pub portfolio: Option<Arc<Mutex<Portfolio>>>,
}

impl<Data, Strategy, Execution, Portfolio> ArbTraderBuilder<Data, Strategy, Execution, Portfolio>
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

    pub fn market(self, value: Vec<Market>) -> Self {
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

    pub fn build(self) -> Result<ArbTrader<Data, Strategy, Execution, Portfolio>, EngineError> {
        Ok(ArbTrader {
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

/*
Standards:
- key: asset_exchange in lowercase

Mutex {
    data: ArbPortfolio {
        engine_id: 4f6e26ab-daab-496e-ad87-3496397011fd,
        exchanges: [
            BinanceSpot,
            PoloniexSpot,
        ],
        repository: InMemoryRepository2 {
            open_positions: {},
            closed_positions: {},
            current_balance: {
                BalanceId2(
                    "USDCE_binancespot",
                ): Balance {
                    time: 2024-11-10T02:07:27.375456Z,
                    total: 0.0,
                    available: 1.0,
                },
                BalanceId2(
                    "USDT_poloniexspot",
                ): Balance {
                    time: 2024-11-10T02:07:27.647360Z,
                    total: 0.0,
                    available: 8.97234541632,
                },
                BalanceId2(
                    "OP_poloniexspot",
                ): Balance {
                    time: 2024-11-10T02:07:27.647363Z,
                    total: 0.0,
                    available: 8.56e-5,
                },
                BalanceId2(
                    "OP_binancespot",
                ): Balance {
                    time: 2024-11-10T02:07:27.375457Z,
                    total: 0.0,
                    available: 0.3115208,
                },
                BalanceId2(
                    "USDT_binancespot",
                ): Balance {
                    time: 2024-11-10T02:07:27.375452Z,
                    total: 0.0,
                    available: 14.76940179,
                },
            },
        },
        allocator: DefaultAllocator {
            default_order_value: 100.0,
        },
    },
}
##############################
order --> OrderEvent {
    time: 2024-11-10T02:07:30.650301Z,
    exchange: PoloniexSpot,
    instrument: Instrument {
        base: "op",
        quote: "usdt",
    },
    market_meta: MarketMeta {
        close: 1.6298,
        time: 2024-11-10T02:07:30.650275Z,
    },
    decision: Long,
    quantity: 61.3572,
    order_type: Limit,
}
##############################
fill --> FillEvent {
    time: 2024-11-10T02:07:30.650400Z,
    exchange: PoloniexSpot,
    instrument: Instrument {
        base: "op",
        quote: "usdt",
    },
    market_meta: MarketMeta {
        close: 1.6298,
        time: 2024-11-10T02:07:30.650275Z,
    },
    decision: Long,
    quantity: 61.3572,
    fill_value_gross: 99.99996456,
    fees: Fees {
        exchange: 0.9999996456,
        slippage: 4.999998228,
        network: 0.0,
    },
}
>>>>> SELF.REPOSITORY <<<<<<
InMemoryRepository2 {
    open_positions: {
        "4f6e26ab-daab-496e-ad87-3496397011fd_poloniexspot_opusdt": Position {
            position_id: "4f6e26ab-daab-496e-ad87-3496397011fd_poloniexspot_opusdt",
            meta: PositionMeta {
                enter_time: 2024-11-10T02:07:30.650275Z,
                update_time: 2024-11-10T02:07:30.650400Z,
                exit_balance: None,
            },
            exchange: PoloniexSpot,
            instrument: Instrument {
                base: "op",
                quote: "usdt",
            },
            side: Buy,
            quantity: 61.3572,
            enter_fees: Fees {
                exchange: 0.9999996456,
                slippage: 4.999998228,
                network: 0.0,
            },
            enter_fees_total: 5.9999978736,
            enter_avg_price_gross: 1.6298,
            enter_value_gross: 99.99996456,
            exit_fees: Fees {
                exchange: 0.0,
                slippage: 0.0,
                network: 0.0,
            },
            exit_fees_total: 0.0,
            exit_avg_price_gross: 0.0,
            exit_value_gross: 0.0,
            current_symbol_price: 1.6298,
            current_value_gross: 99.99996456,
            unrealised_profit_loss: -11.9999957472,
            realised_profit_loss: 0.0,
        },
    },
    closed_positions: {},
    current_balance: {
        BalanceId2(
            "USDCE_binancespot",
        ): Balance {
            time: 2024-11-10T02:07:27.375456Z,
            total: 0.0,
            available: 1.0,
        },
        BalanceId2(
            "USDT_poloniexspot",
        ): Balance {
            time: 2024-11-10T02:07:27.647360Z,
            total: 0.0,
            available: 8.97234541632,
        },
        BalanceId2(
            "OP_poloniexspot",
        ): Balance {
            time: 2024-11-10T02:07:27.647363Z,
            total: 0.0,
            available: 8.56e-5,
        },
        BalanceId2(
            "OP_binancespot",
        ): Balance {
            time: 2024-11-10T02:07:27.375457Z,
            total: 0.0,
            available: 0.3115208,
        },
        BalanceId2(
            "USDT_binancespot",
        ): Balance {
            time: 2024-11-10T02:07:27.375452Z,
            total: 0.0,
            available: 14.76940179,
        },
    },
}
##############################
fill event --> [
    PositionNew(
        Position {
            position_id: "4f6e26ab-daab-496e-ad87-3496397011fd_poloniexspot_opusdt",
            meta: PositionMeta {
                enter_time: 2024-11-10T02:07:30.650275Z,
                update_time: 2024-11-10T02:07:30.650400Z,
                exit_balance: None,
            },
            exchange: PoloniexSpot,
            instrument: Instrument {
                base: "op",
                quote: "usdt",
            },
            side: Buy,
            quantity: 61.3572,
            enter_fees: Fees {
                exchange: 0.9999996456,
                slippage: 4.999998228,
                network: 0.0,
            },
            enter_fees_total: 5.9999978736,
            enter_avg_price_gross: 1.6298,
            enter_value_gross: 99.99996456,
            exit_fees: Fees {
                exchange: 0.0,
                slippage: 0.0,
                network: 0.0,
            },
            exit_fees_total: 0.0,
            exit_avg_price_gross: 0.0,
            exit_value_gross: 0.0,
            current_symbol_price: 1.6298,
            current_value_gross: 99.99996456,
            unrealised_profit_loss: -11.9999957472,
            realised_profit_loss: 0.0,
        },
    ),
]
##############################
order --> OrderEvent {
    time: 2024-11-10T02:07:31.042574Z,
    exchange: BinanceSpot,
    instrument: Instrument {
        base: "op",
        quote: "usdt",
    },
    market_meta: MarketMeta {
        close: 1.629,
        time: 2024-11-10T02:07:31.042549Z,
    },
    decision: Long,
    quantity: 61.3873,
    order_type: Limit,
}
##############################
fill --> FillEvent {
    time: 2024-11-10T02:07:31.042624Z,
    exchange: BinanceSpot,
    instrument: Instrument {
        base: "op",
        quote: "usdt",
    },
    market_meta: MarketMeta {
        close: 1.629,
        time: 2024-11-10T02:07:31.042549Z,
    },
    decision: Long,
    quantity: 61.3873,
    fill_value_gross: 99.99991170000001,
    fees: Fees {
        exchange: 0.9999991170000001,
        slippage: 4.999995585000001,
        network: 0.0,
    },
}
>>>>> SELF.REPOSITORY <<<<<<
InMemoryRepository2 {
    open_positions: {
        "4f6e26ab-daab-496e-ad87-3496397011fd_poloniexspot_opusdt": Position {
            position_id: "4f6e26ab-daab-496e-ad87-3496397011fd_poloniexspot_opusdt",
            meta: PositionMeta {
                enter_time: 2024-11-10T02:07:30.650275Z,
                update_time: 2024-11-10T02:07:31.040907Z,
                exit_balance: None,
            },
            exchange: PoloniexSpot,
            instrument: Instrument {
                base: "op",
                quote: "usdt",
            },
            side: Buy,
            quantity: 61.3572,
            enter_fees: Fees {
                exchange: 0.9999996456,
                slippage: 4.999998228,
                network: 0.0,
            },
            enter_fees_total: 5.9999978736,
            enter_avg_price_gross: 1.6298,
            enter_value_gross: 99.99996456,
            exit_fees: Fees {
                exchange: 0.0,
                slippage: 0.0,
                network: 0.0,
            },
            exit_fees_total: 0.0,
            exit_avg_price_gross: 0.0,
            exit_value_gross: 0.0,
            current_symbol_price: 1.6319224032397628,
            current_value_gross: 100.13018928006277,
            unrealised_profit_loss: -11.86977102713723,
            realised_profit_loss: 0.0,
        },
        "4f6e26ab-daab-496e-ad87-3496397011fd_binancespot_opusdt": Position {
            position_id: "4f6e26ab-daab-496e-ad87-3496397011fd_binancespot_opusdt",
            meta: PositionMeta {
                enter_time: 2024-11-10T02:07:31.042549Z,
                update_time: 2024-11-10T02:07:31.042624Z,
                exit_balance: None,
            },
            exchange: BinanceSpot,
            instrument: Instrument {
                base: "op",
                quote: "usdt",
            },
            side: Buy,
            quantity: 61.3873,
            enter_fees: Fees {
                exchange: 0.9999991170000001,
                slippage: 4.999995585000001,
                network: 0.0,
            },
            enter_fees_total: 5.999994702,
            enter_avg_price_gross: 1.6290000000000002,
            enter_value_gross: 99.99991170000001,
            exit_fees: Fees {
                exchange: 0.0,
                slippage: 0.0,
                network: 0.0,
            },
            exit_fees_total: 0.0,
            exit_avg_price_gross: 0.0,
            exit_value_gross: 0.0,
            current_symbol_price: 1.6290000000000002,
            current_value_gross: 99.99991170000001,
            unrealised_profit_loss: -11.999989404,
            realised_profit_loss: 0.0,
        },
    },
    closed_positions: {},
    current_balance: {
        BalanceId2(
            "USDCE_binancespot",
        ): Balance {
            time: 2024-11-10T02:07:27.375456Z,
            total: 0.0,
            available: 1.0,
        },
        BalanceId2(
            "USDT_poloniexspot",
        ): Balance {
            time: 2024-11-10T02:07:27.647360Z,
            total: 0.0,
            available: 8.97234541632,
        },
        BalanceId2(
            "OP_poloniexspot",
        ): Balance {
            time: 2024-11-10T02:07:27.647363Z,
            total: 0.0,
            available: 8.56e-5,
        },
        BalanceId2(
            "OP_binancespot",
        ): Balance {
            time: 2024-11-10T02:07:27.375457Z,
            total: 0.0,
            available: 0.3115208,
        },
        BalanceId2(
            "USDT_binancespot",
        ): Balance {
            time: 2024-11-10T02:07:27.375452Z,
            total: 0.0,
            available: 14.76940179,
        },
    },
}
##############################
fill event --> [
    PositionNew(
        Position {
            position_id: "4f6e26ab-daab-496e-ad87-3496397011fd_binancespot_opusdt",
            meta: PositionMeta {
                enter_time: 2024-11-10T02:07:31.042549Z,
                update_time: 2024-11-10T02:07:31.042624Z,
                exit_balance: None,
            },
            exchange: BinanceSpot,
            instrument: Instrument {
                base: "op",
                quote: "usdt",
            },
            side: Buy,
            quantity: 61.3873,
            enter_fees: Fees {
                exchange: 0.9999991170000001,
                slippage: 4.999995585000001,
                network: 0.0,
            },
            enter_fees_total: 5.999994702,
            enter_avg_price_gross: 1.6290000000000002,
            enter_value_gross: 99.99991170000001,
            exit_fees: Fees {
                exchange: 0.0,
                slippage: 0.0,
                network: 0.0,
            },
            exit_fees_total: 0.0,
            exit_avg_price_gross: 0.0,
            exit_value_gross: 0.0,
            current_symbol_price: 1.6290000000000002,
            current_value_gross: 99.99991170000001,
            unrealised_profit_loss: -11.999989404,
            realised_profit_loss: 0.0,
        },
    ),
]
*/