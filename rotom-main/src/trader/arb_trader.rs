use async_trait::async_trait;
use parking_lot::Mutex;
use rotom_data::{
    event_models::market_event::{DataKind, MarketEvent},
    Feed, Market, MarketGenerator,
};
use rotom_oms::{
    event::{Event, EventTx, MessageTransmitter},
    exchange::ExecutionClient,
    model::{
        account_data::AccountData,
        order::{ExecutionRequest, OpenOrder},
    },
    portfolio::portfolio_type::{FillUpdater, MarketUpdater, OrderGenerator},
};
use rotom_strategy::{SignalForceExit, SignalGenerator};
use std::{collections::VecDeque, sync::Arc};
use tokio::sync::mpsc;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::engine::{error::EngineError, Command};

use super::TraderRun;

/*----- */
// Spot Arb Trader - Metadata info
/*----- */
pub struct ArbTraderMetaData {
    pub taker_buy_liquid: bool,
    pub taker_sell_liquid: bool,
    pub transfering_to_illiquid: bool,
    pub maker_buy_illiquid: bool,
    pub maker_sell_illiquid: bool,
    pub transfering_to_liquid: bool,
}

/*----- */
// Spot Arb Trader Lego
/*----- */
#[derive(Debug)]
pub struct SpotArbTraderLego<Data, Strategy, LiquidExchange, IlliquidExchange, Portfolio>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
    Strategy: SignalGenerator,
    LiquidExchange: ExecutionClient,
    IlliquidExchange: ExecutionClient,
    Portfolio: MarketUpdater + OrderGenerator + FillUpdater,
{
    pub engine_id: Uuid,
    pub command_rx: mpsc::Receiver<Command>,
    pub event_tx: EventTx,
    pub markets: Vec<Market>,
    pub data: Data,
    pub stategy: Strategy,
    pub liquid_exchange: LiquidExchange,
    pub illiquid_exchange: IlliquidExchange,
    pub send_order_tx: mpsc::UnboundedSender<ExecutionRequest>,
    pub order_update_rx: mpsc::Receiver<AccountData>,
    pub porfolio: Arc<Mutex<Portfolio>>,
}

/*----- */
// Spot Arb Trader
/*----- */
#[derive(Debug)]
pub struct SpotArbTrader<Data, Strategy, LiquidExchange, IlliquidExchange, Portfolio>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
    Strategy: SignalGenerator,
    LiquidExchange: ExecutionClient,
    IlliquidExchange: ExecutionClient,
    Portfolio: MarketUpdater + OrderGenerator,
{
    engine_id: Uuid,
    command_rx: mpsc::Receiver<Command>,
    event_tx: EventTx,
    markets: Vec<Market>,
    data: Data,
    stategy: Strategy,
    liquid_exchange: LiquidExchange,
    illiquid_exchange: IlliquidExchange,
    order_update_rx: mpsc::Receiver<AccountData>,
    event_queue: VecDeque<Event>,
    portfolio: Arc<Mutex<Portfolio>>,
}

impl<Data, Strategy, LiquidExchange, IlliquidExchange, Portfolio>
    SpotArbTrader<Data, Strategy, LiquidExchange, IlliquidExchange, Portfolio>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
    Strategy: SignalGenerator,
    LiquidExchange: ExecutionClient,
    IlliquidExchange: ExecutionClient,
    Portfolio: MarketUpdater + OrderGenerator + FillUpdater,
{
    pub fn new(
        lego: SpotArbTraderLego<Data, Strategy, LiquidExchange, IlliquidExchange, Portfolio>,
    ) -> Self {
        Self {
            engine_id: lego.engine_id,
            command_rx: lego.command_rx,
            event_tx: lego.event_tx,
            markets: lego.markets,
            data: lego.data,
            stategy: lego.stategy,
            liquid_exchange: lego.liquid_exchange,
            illiquid_exchange: lego.illiquid_exchange,
            order_update_rx: lego.order_update_rx,
            event_queue: VecDeque::with_capacity(4),
            portfolio: lego.porfolio,
        }
    }

    pub fn builder(
    ) -> SpotArbTraderBuilder<Data, Strategy, LiquidExchange, IlliquidExchange, Portfolio> {
        SpotArbTraderBuilder::new()
    }
}

/*----- */
// Impl Trader trait for Single Market Trader
/*----- */
#[async_trait]
impl<Data, Strategy, LiquidExchange, IlliquidExchange, Portfolio> TraderRun
    for SpotArbTrader<Data, Strategy, LiquidExchange, IlliquidExchange, Portfolio>
where
    Data: MarketGenerator<MarketEvent<DataKind>> + Send,
    Strategy: SignalGenerator + Send,
    LiquidExchange: ExecutionClient + Send,
    IlliquidExchange: ExecutionClient + Send,
    Portfolio: MarketUpdater + OrderGenerator + FillUpdater + Send,
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

    async fn run(mut self) {
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
                            // println!("##############################");
                            // println!("order --> {:#?}", order);
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
                        // let new_order = self
                        //     .illiquid_exchange
                        //     .open_order(OpenOrder::from(&order))
                        //     .await;

                        // let new_order2 = self
                        //     .liquid_exchange
                        //     .open_order(OpenOrder::from(&order))
                        //     .await;
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

            // Check for account data event and break out trading loop if the
            // rx disconnects
            match self.order_update_rx.try_recv() {
                Ok(account_data) => match account_data {
                    AccountData::Order(order) => {
                        println!("AccountData: {:#?}", order);
                        self.event_queue.push_back(Event::AccountDataOrder(order))
                    }
                    AccountData::BalanceVec(balances) => {
                        println!("AccountData: {:#?}", balances);
                        self.event_queue
                            .push_back(Event::AccountDataBalanceVec(balances));
                    }

                    AccountData::BalanceDelta(balance_delta) => {
                        println!("AccountData: {:#?}", balance_delta);
                        self.event_queue
                            .push_back(Event::AccountDataBalanceDelta(balance_delta))
                    }

                    AccountData::Balance(balance) => {
                        println!("AccountData: {:#?}", balance);
                        self.event_queue
                            .push_back(Event::AccountDataBalance(balance))
                    }
                },
                Err(error) => {
                    if error == mpsc::error::TryRecvError::Disconnected {
                        warn!(
                            message = "Order update rx for ArbTrader disconnected",
                            asset_one  = %self.markets[0],
                            asset_two  = %self.markets[1]
                        );
                        break 'trading;
                    }
                }
            }
        }

        debug!(
            engine_id = &*self.engine_id.to_string(),
            market = &*format!("{:?}", self.markets),
            message = "Trader trading loop stopped"
        )
    }
}

/*----- */
// Single Market Trader builder
/*----- */
#[derive(Debug, Default)]
pub struct SpotArbTraderBuilder<Data, Strategy, LiquidExchange, IlliquidExchange, Portfolio>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
    Strategy: SignalGenerator,
    LiquidExchange: ExecutionClient,
    IlliquidExchange: ExecutionClient,
    Portfolio: MarketUpdater + OrderGenerator + FillUpdater,
{
    pub engine_id: Option<Uuid>,
    pub command_rx: Option<mpsc::Receiver<Command>>,
    pub event_tx: Option<EventTx>,
    pub markets: Option<Vec<Market>>,
    pub data: Option<Data>,
    pub strategy: Option<Strategy>,
    pub liquid_exchange: Option<LiquidExchange>,
    pub illiquid_exchange: Option<IlliquidExchange>,
    pub send_order_tx: Option<mpsc::UnboundedSender<ExecutionRequest>>,
    pub order_update_rx: Option<mpsc::Receiver<AccountData>>,
    pub portfolio: Option<Arc<Mutex<Portfolio>>>,
}

impl<Data, Strategy, LiquidExchange, IlliquidExchange, Portfolio>
    SpotArbTraderBuilder<Data, Strategy, LiquidExchange, IlliquidExchange, Portfolio>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
    Strategy: SignalGenerator,
    LiquidExchange: ExecutionClient,
    IlliquidExchange: ExecutionClient,
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
            liquid_exchange: None,
            illiquid_exchange: None,
            send_order_tx: None,
            order_update_rx: None,
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

    pub fn liquid_exchange(self, value: LiquidExchange) -> Self {
        Self {
            liquid_exchange: Some(value),
            ..self
        }
    }

    pub fn illiquid_exchange(self, value: IlliquidExchange) -> Self {
        Self {
            illiquid_exchange: Some(value),
            ..self
        }
    }

    pub fn send_order_tx(self, value: mpsc::UnboundedSender<ExecutionRequest>) -> Self {
        Self {
            send_order_tx: Some(value),
            ..self
        }
    }

    pub fn order_update_rx(self, value: mpsc::Receiver<AccountData>) -> Self {
        Self {
            order_update_rx: Some(value),
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
    ) -> Result<
        SpotArbTrader<Data, Strategy, LiquidExchange, IlliquidExchange, Portfolio>,
        EngineError,
    > {
        Ok(SpotArbTrader {
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
            liquid_exchange: self
                .liquid_exchange
                .ok_or(EngineError::BuilderIncomplete("liquid_exchange"))?,
            illiquid_exchange: self
                .illiquid_exchange
                .ok_or(EngineError::BuilderIncomplete("illiquid_exchange"))?,
            order_update_rx: self
                .order_update_rx
                .ok_or(EngineError::BuilderIncomplete("order_update_rx"))?,
            event_queue: VecDeque::with_capacity(2),
            portfolio: self
                .portfolio
                .ok_or(EngineError::BuilderIncomplete("portfolio"))?,
        })
    }
}
