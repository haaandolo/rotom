use async_trait::async_trait;
use parking_lot::Mutex;
use rotom_data::{
    event_models::market_event::{DataKind, MarketEvent},
    shared::subscription_models::ExchangeId,
    Feed, Market, MarketGenerator,
};
use rotom_oms::{
    event::{Event, EventTx, MessageTransmitter},
    exchange::ExecutionClient,
    model::{
        account_data::AccountData,
        order::{ExecutionRequest, OpenOrder, OrderEvent, OrderState, WalletTransfer},
        OrderKind,
    },
    portfolio::portfolio_type::{
        spot_portfolio::SpotPortfolio, FillUpdater, MarketUpdater, OrderGenerator,
    },
};
use rotom_strategy::{Decision, SignalForceExit, SignalGenerator};
use std::{collections::VecDeque, sync::Arc};
use tokio::sync::mpsc;
use tracing::{debug, error, warn};
use uuid::Uuid;

use crate::engine::{error::EngineError, Command};

use super::TraderRun;

/*----- */
// Spot Arb Trader - Metadata info
/*----- */
#[derive(Debug, PartialEq, Clone, Default)]
pub enum SpotArbTraderExecutionState {
    #[default]
    NoPosition,
    TakerBuyLiquid,
    TakerSellLiquid,
    TransferToLiquid,
    MakerBuyIlliquid,
    MakerSellIlliquid,
    TransferToIlliquid,
}

#[derive(Debug, Default)]
pub struct SpotArbTraderMetaData {
    pub order: Option<OrderEvent>,
    pub execution_state: SpotArbTraderExecutionState,
    pub liquid_deposit_address: String,
    pub illiquid_deposit_address: String,
}

/*----- */
// Spot Arb Trader Lego
/*----- */
#[derive(Debug)]
pub struct SpotArbTraderLego<Data, Strategy, LiquidExchange, IlliquidExchange>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
    Strategy: SignalGenerator,
    LiquidExchange: ExecutionClient,
    IlliquidExchange: ExecutionClient,
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
    pub porfolio: Arc<Mutex<SpotPortfolio>>,
    pub meta_data: SpotArbTraderMetaData,
}

/*----- */
// Spot Arb Trader
/*----- */
#[derive(Debug)]
pub struct SpotArbTrader<Data, Strategy, LiquidExchange, IlliquidExchange>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
    Strategy: SignalGenerator,
    LiquidExchange: ExecutionClient,
    IlliquidExchange: ExecutionClient,
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
    portfolio: Arc<Mutex<SpotPortfolio>>,
    meta_data: SpotArbTraderMetaData,
}

impl<Data, Strategy, LiquidExchange, IlliquidExchange>
    SpotArbTrader<Data, Strategy, LiquidExchange, IlliquidExchange>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
    Strategy: SignalGenerator,
    LiquidExchange: ExecutionClient,
    IlliquidExchange: ExecutionClient,
{
    pub fn new(lego: SpotArbTraderLego<Data, Strategy, LiquidExchange, IlliquidExchange>) -> Self {
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
            meta_data: lego.meta_data,
        }
    }

    pub fn builder() -> SpotArbTraderBuilder<Data, Strategy, LiquidExchange, IlliquidExchange> {
        SpotArbTraderBuilder::new()
    }

    pub async fn process_new_order(&mut self) {
        if let Some(order) = &mut self.meta_data.order {
            if order.get_exchange() == LiquidExchange::CLIENT {
                // order.order_kind = OrderKind::Market;
                let liquid_new_order = self
                    .liquid_exchange
                    .open_order(OpenOrder::from(order))
                    .await;
                self.meta_data.execution_state = SpotArbTraderExecutionState::TakerBuyLiquid;
                println!("########################");
                println!("### Liquid new order ###");
                println!("########################");
                println!("{:#?}", liquid_new_order);
            } else {
                // order.order_kind = OrderKind::Limit;
                let illiquid_new_order = self
                    .illiquid_exchange
                    .open_order(OpenOrder::from(order))
                    .await;
                self.meta_data.execution_state = SpotArbTraderExecutionState::MakerBuyIlliquid;
                println!("########################");
                println!("### Illiquid new order ###");
                println!("########################");
                println!("{:#?}", illiquid_new_order);
            }
        }
    }

    fn get_execution_state(&self) {
        match self.meta_data.execution_state {
            SpotArbTraderExecutionState::NoPosition => {}
            SpotArbTraderExecutionState::MakerBuyIlliquid => {}
            SpotArbTraderExecutionState::MakerSellIlliquid => {}
            SpotArbTraderExecutionState::TransferToIlliquid => {}
            SpotArbTraderExecutionState::TakerBuyLiquid => {}
            SpotArbTraderExecutionState::TakerSellLiquid => {}
            SpotArbTraderExecutionState::TransferToLiquid => {}
        }
    }

    pub async fn process_existing_order(&mut self) {
        unimplemented!()
    }
}

/*----- */
// Impl Trader trait for Single Market Trader
/*----- */
#[async_trait]
impl<Data, Strategy, LiquidExchange, IlliquidExchange> TraderRun
    for SpotArbTrader<Data, Strategy, LiquidExchange, IlliquidExchange>
where
    Data: MarketGenerator<MarketEvent<DataKind>> + Send + Sync,
    Strategy: SignalGenerator + Send + Sync,
    LiquidExchange: ExecutionClient + Send + Sync,
    IlliquidExchange: ExecutionClient + Send + Sync,
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
            /*---------------------- 0.Check for remote cmd --------------------- */
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

            /*-------------------- 1.Get Market Event Update -------------------- */
            match self.data.next() {
                Feed::Next(market) => {
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

            /*--------------------- 2.Process Event Loop ------------------------ */
            // The event loop is a while loop that will break if there are no more
            // events in the queue. If we send an order in the  event loop, we will
            // receieve the update in step 3. Note, since we are awaiting for a
            // response in this loop, we should get the update corresponding to the
            // order next up.
            while let Some(event) = self.event_queue.pop_front() {
                match event {
                    Event::Market(market_event) => {
                        if let Some(signal) = self.stategy.generate_signal(&market_event) {
                            // println!("##############################");
                            // println!("signal --> {:#?}", signal);
                            // self.event_tx.send(Event::Signal(signal.clone()));
                            self.event_queue.push_back(Event::Signal(signal))
                        }

                        if let Some(position_update) = self
                            .portfolio
                            .lock()
                            .update_from_market2(&market_event)
                            .expect("Failed to update Portfolio from MarketEvent")
                        {
                            // println!("##############################");
                            // println!("position update --> {:#?}", position_update);
                            // self.event_tx.send(Event::PositionUpdate(position_update));
                        }
                    }
                    Event::Signal(signal) => {
                        if let Some(order) = self
                            .portfolio
                            .lock()
                            .generate_order2(&signal)
                            .expect("Failed to generate order")
                        {
                            // println!("##############################");
                            // println!("order --> {:#?}", order);
                            // self.event_tx.send(Event::OrderNew(order.clone()));
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
                            // self.event_tx.send(Event::OrderNew(order.clone()));
                            self.event_queue.push_back(Event::OrderNew(order));
                        }
                    }
                    Event::OrderNew(mut new_order) => match &self.meta_data.order {
                        Some(_) => {
                            //  self.process_existing_order().await;
                        }
                        None => {
                            // // Del
                            new_order.exchange = ExchangeId::PoloniexSpot;
                            new_order.original_quantity = 5.0;
                            new_order.market_meta.close = 1.8300;
                            new_order.order_kind = OrderKind::Market;
                            // // Del

                            // self.meta_data.order = Some(new_order);
                            // self.process_new_order().await;
                        }
                    },
                    _ => {}
                }
            }

            /*-------------------- 3.Process Account Data update ---------------- */
            // If we interacted with the exchange in anyway in the second step, we
            // will get the order update in this section.

            // todo: turn this into a while let loop??
            'account_data_loop: loop {
                match self.order_update_rx.try_recv() {
                    Ok(account_data) => match account_data {
                        AccountData::Order(account_data_order_update) => {
                            println!("AccountDataOrder: {:#?}", account_data_order_update);
                            // self.event_queue.push_back(Event::AccountDataOrder(account_data_order_update));
                            if let Some(order) = &mut self.meta_data.order {
                                // When account data order update comes, update OrderEvent state first
                                order.update_order_from_account_data_stream(
                                    &account_data_order_update,
                                );

                                println!("### Order event ###");
                                println!("{:#?}", order);

                                // Then update the portfolio position state with updated OrderEvent
                                let _ = self.portfolio.lock().update_from_fill2(order);

                                // Check if order is filled
                                if order.is_order_filled() {
                                    /*----- transfer to illiquid ----- */
                                    if order.exchange == LiquidExchange::CLIENT
                                        && self.meta_data.execution_state
                                            == SpotArbTraderExecutionState::TakerBuyLiquid
                                    {
                                        'transfer_to_illiquid: loop {
                                            let wallet_transfer_request = WalletTransfer::new(
                                                order.instrument.base.clone(),
                                                self.meta_data.illiquid_deposit_address.clone(),
                                                None,
                                                order.cumulative_quantity - order.fees,
                                            );

                                            println!("########################");
                                            println!(
                                                "### Wallet transfer request - to illiquid ###"
                                            );
                                            println!("########################");
                                            println!("{:#?}", wallet_transfer_request);

                                            match self
                                                .liquid_exchange
                                                .wallet_transfer(wallet_transfer_request)
                                                .await
                                            {
                                                Ok(_) => break 'transfer_to_illiquid,
                                                Err(error) => {
                                                    error!(
                                                        message = "error when sending request to exchange",
                                                        error = %error
                                                    );
                                                    continue;
                                                }
                                            }
                                        }

                                        self.meta_data.execution_state =
                                            SpotArbTraderExecutionState::TransferToIlliquid;
                                    }

                                    /*----- transfer to liquid ----- */
                                    if order.exchange == IlliquidExchange::CLIENT
                                        && self.meta_data.execution_state
                                            == SpotArbTraderExecutionState::MakerBuyIlliquid
                                    {
                                        'transfer_to_liquid: loop {
                                            let wallet_transfer_request = WalletTransfer::new(
                                                order.instrument.base.clone(),
                                                self.meta_data.liquid_deposit_address.clone(),
                                                None,
                                                order.cumulative_quantity - order.fees,
                                            );
                                            println!("########################");
                                            println!("### Wallet transfer request - to liquid ###");
                                            println!("########################");
                                            println!("{:#?}", wallet_transfer_request);

                                            match self
                                                .illiquid_exchange
                                                .wallet_transfer(wallet_transfer_request)
                                                .await
                                            {
                                                Ok(_) => break 'transfer_to_liquid,
                                                Err(error) => {
                                                    error!(
                                                        message = "error when sending request to exchange",
                                                        error = %error
                                                    );
                                                    continue;
                                                }
                                            }
                                        }

                                        self.meta_data.execution_state =
                                            SpotArbTraderExecutionState::TransferToLiquid;
                                    }
                                }
                                // println!("####### ORDER UPDATED ##########");
                                // println!("{:#?}", order)
                            }
                        }
                        AccountData::Balance(balance_update) => {
                            println!("AccountDataBalance: {:#?}", balance_update);
                            if let Some(ref order) = self.meta_data.order {
                                if order.is_order_filled() {
                                    /*----- coin has been transfered to liquid exchange ----- */
                                    if balance_update.exchange == LiquidExchange::CLIENT
                                        && self.meta_data.execution_state
                                            == SpotArbTraderExecutionState::TransferToLiquid
                                    {
                                        let mut testing_sell_req = OpenOrder::from(order);
                                        testing_sell_req.decision = Decision::Short;
                                        testing_sell_req.order_kind = OrderKind::Market;
                                        testing_sell_req.quantity = balance_update.balance.total;

                                        println!("########################");
                                        println!(
                                            "#### testing sell after transfer to liquid ###"
                                        );
                                        println!("########################");
                                        println!("req --> {:#?}", testing_sell_req);

                                        let res = self
                                            .liquid_exchange
                                            .open_order(testing_sell_req)
                                            .await;

                                        println!("res ---> {:#?}", res);
                                    }

                                    /*----- coin has been transfered to illiquid exchange ----- */
                                    if balance_update.exchange == IlliquidExchange::CLIENT
                                        && self.meta_data.execution_state
                                            == SpotArbTraderExecutionState::TransferToIlliquid
                                    {
                                        let mut testing_sell_req = OpenOrder::from(order);
                                        testing_sell_req.decision = Decision::Short;
                                        testing_sell_req.order_kind = OrderKind::Market;
                                        testing_sell_req.price = 1.80;
                                        testing_sell_req.quantity = balance_update.balance.total;

                                        println!("########################");
                                        println!(
                                            "#### testing sell after transfer to illiquid ###"
                                        );
                                        println!("########################");
                                        println!("req --> {:#?}", testing_sell_req);

                                        let res = self
                                            .illiquid_exchange
                                            .open_order(testing_sell_req)
                                            .await;

                                        println!("res ---> {:#?}", res);
                                    }
                                }
                            }
                        }
                        AccountData::BalanceDelta(balance_delta) => {
                            println!("AccountDataBalanceDelta: {:#?}", balance_delta);
                        }
                        // AccountData::BalanceVec get split into individual balances
                        // and gets sent to corresponding trader so this one is not needed
                        _ => {}
                    },
                    Err(error) => match error {
                        mpsc::error::TryRecvError::Empty => break 'account_data_loop,
                        mpsc::error::TryRecvError::Disconnected => {
                            warn!(
                                message = "Order update rx for ArbTrader disconnected",
                                asset_one  = %self.markets[0],
                                asset_two  = %self.markets[1]
                            );
                            break 'trading;
                        }
                    },
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
pub struct SpotArbTraderBuilder<Data, Strategy, LiquidExchange, IlliquidExchange>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
    Strategy: SignalGenerator,
    LiquidExchange: ExecutionClient,
    IlliquidExchange: ExecutionClient,
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
    pub portfolio: Option<Arc<Mutex<SpotPortfolio>>>,
    pub meta_data: Option<SpotArbTraderMetaData>,
}

impl<Data, Strategy, LiquidExchange, IlliquidExchange>
    SpotArbTraderBuilder<Data, Strategy, LiquidExchange, IlliquidExchange>
where
    Data: MarketGenerator<MarketEvent<DataKind>>,
    Strategy: SignalGenerator,
    LiquidExchange: ExecutionClient,
    IlliquidExchange: ExecutionClient,
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
            meta_data: None,
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

    pub fn portfolio(self, value: Arc<Mutex<SpotPortfolio>>) -> Self {
        Self {
            portfolio: Some(value),
            ..self
        }
    }

    pub fn meta_data(self, value: SpotArbTraderMetaData) -> Self {
        Self {
            meta_data: Some(value),
            ..self
        }
    }

    pub fn build(
        self,
    ) -> Result<SpotArbTrader<Data, Strategy, LiquidExchange, IlliquidExchange>, EngineError> {
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
            meta_data: self
                .meta_data
                .ok_or(EngineError::BuilderIncomplete("meta_data"))?,
        })
    }
}

/*
### Binance ###
arb portfolio: Mutex {
    data: SpotPortfolio {
        engine_id: 37fd5e92-a909-4638-a539-f40262c761a7,
        exchanges: [
            BinanceSpot,
            PoloniexSpot,
        ],
        repository: SpotInMemoryRepository {
            open_positions: {},
            closed_positions: {},
            current_balance: {
                SpotBalanceId(
                    "binancespot_usdt",
                ): Balance {
                    total: 9.39136955,
                    available: 0.0,
                },
                SpotBalanceId(
                    "binancespot_arb",
                ): Balance {
                    total: 0.0894,
                    available: 0.0,
                },
                SpotBalanceId(
                    "poloniexspot_usdt",
                ): Balance {
                    total: 11.06022807796,
                    available: 0.0,
                },
                SpotBalanceId(
                    "poloniexspot_op",
                ): Balance {
                    total: 8.04e-5,
                    available: 0.0,
                },
                SpotBalanceId(
                    "binancespot_op",
                ): Balance {
                    total: 0.00011,
                    available: 0.0,
                },
                SpotBalanceId(
                    "binancespot_usdce",
                ): Balance {
                    total: 1.0,
                    available: 0.0,
                },
            },
        },
        allocator: SpotArbAllocator,
    },
}

AccountDataOrder: AccountDataOrder {
    exchange: BinanceSpot,
    client_order_id: "T01L6y9R4FvUKe15X8Si4n",
    asset: "OPUSDT",
    price: 0.0,
    quantity: 0.0,
    status: New,
    execution_time: 2024-12-22T02:55:33.273Z,
    side: Buy,
    fee: 0.0,
    filled_gross: 0.0,
}
####### ORDER BEFORE UPDATE ##########
OrderEvent {
    order_request_time: 2024-12-22T02:55:32.950258Z,
    exchange: BinanceSpot,
    client_order_id: None,
    instrument: Instrument {
        base: "op",
        quote: "usdt",
    },
    market_meta: MarketMeta {
        close: 1.746,
        time: 2024-12-22T02:55:32.950255Z,
    },
    decision: Long,
    original_quantity: 4.0,
    cumulative_quantity: 0.0,
    order_kind: Limit,
    exchange_order_status: None,
    internal_order_state: InTransit,
    filled_gross: 0.0,
    enter_avg_price: 0.0,
    fees: 0.0,
    last_execution_time: None,
}
############ IN NONE ############
############ UPDATE FROM FILL 2 ############
SpotPortfolio {
    engine_id: 37fd5e92-a909-4638-a539-f40262c761a7,
    exchanges: [
        BinanceSpot,
        PoloniexSpot,
    ],
    repository: SpotInMemoryRepository {
        open_positions: {
            ExchangeAssetId(
                "BINANCESPOT_OPUSDT",
            ): Position2 {
                order_request_time: 2024-12-22T02:55:32.950258Z,
                last_execution_time: 2024-12-22T02:55:33.273Z,
                position_id: ExchangeAssetId(
                    "BINANCESPOT_OPUSDT",
                ),
                exchange: BinanceSpot,
                asset: AssetFormatted(
                    "OPUSDT",
                ),
                side: Buy,
                quantity: 0.0,
                fees: 0.0,
                current_symbol_price: 0.0,
                current_value_gross: 0.0,
                filled_gross: 0.0,
                enter_avg_price: 0.0,
                enter_value_gross: 0.0,
                unrealised_profit_loss: -0.0,
                realised_profit_loss: 0.0,
            },
        },
        closed_positions: {},
        current_balance: {
            SpotBalanceId(
                "binancespot_usdt",
            ): Balance {
                total: 2.40736955,
                available: 0.0,
            },
            SpotBalanceId(
                "binancespot_arb",
            ): Balance {
                total: 0.0894,
                available: 0.0,
            },
            SpotBalanceId(
                "poloniexspot_usdt",
            ): Balance {
                total: 11.06022807796,
                available: 0.0,
            },
            SpotBalanceId(
                "poloniexspot_op",
            ): Balance {
                total: 8.04e-5,
                available: 0.0,
            },
            SpotBalanceId(
                "binancespot_bnb",
            ): Balance {
                total: 0.0,
                available: 0.0,
            },
            SpotBalanceId(
                "binancespot_op",
            ): Balance {
                total: 0.00011,
                available: 0.0,
            },
            SpotBalanceId(
                "binancespot_usdce",
            ): Balance {
                total: 1.0,
                available: 0.0,
            },
        },
    },
    allocator: SpotArbAllocator,
}
####### ORDER UPDATED ##########
OrderEvent {
    order_request_time: 2024-12-22T02:55:32.950258Z,
    exchange: BinanceSpot,
    client_order_id: Some(
        ClientOrderId(
            "T01L6y9R4FvUKe15X8Si4n",
        ),
    ),
    instrument: Instrument {
        base: "op",
        quote: "usdt",
    },
    market_meta: MarketMeta {
        close: 1.746,
        time: 2024-12-22T02:55:32.950255Z,
    },
    decision: Long,
    original_quantity: 4.0,
    cumulative_quantity: 0.0,
    order_kind: Limit,
    exchange_order_status: Some(
        New,
    ),
    internal_order_state: Open,
    filled_gross: 0.0,
    enter_avg_price: NaN,
    fees: 0.0,
    last_execution_time: Some(
        2024-12-22T02:55:33.273Z,
    ),
}
AccountDataBalance: AccountDataBalance {
    asset: "OP",
    exchange: BinanceSpot,
    balance: Balance {
        total: 0.00011,
        available: 0.0,
    },
}

AccountDataOrder: AccountDataOrder {
    exchange: BinanceSpot,
    client_order_id: "T01L6y9R4FvUKe15X8Si4n",
    asset: "OPUSDT",
    price: 1.746,
    quantity: 4.0,
    status: Filled,
    execution_time: 2024-12-22T02:55:50.400Z,
    side: Buy,
    fee: 0.004,
    filled_gross: 6.984,
}
####### ORDER BEFORE UPDATE ##########
OrderEvent {
    order_request_time: 2024-12-22T02:55:32.950258Z,
    exchange: BinanceSpot,
    client_order_id: Some(
        ClientOrderId(
            "T01L6y9R4FvUKe15X8Si4n",
        ),
    ),
    instrument: Instrument {
        base: "op",
        quote: "usdt",
    },
    market_meta: MarketMeta {
        close: 1.746,
        time: 2024-12-22T02:55:32.950255Z,
    },
    decision: Long,
    original_quantity: 4.0,
    cumulative_quantity: 0.0,
    order_kind: Limit,
    exchange_order_status: Some(
        New,
    ),
    internal_order_state: Open,
    filled_gross: 0.0,
    enter_avg_price: NaN,
    fees: 0.0,
    last_execution_time: Some(
        2024-12-22T02:55:33.273Z,
    ),
}
############ IN SOME ############
############ UPDATE FROM FILL 2 ############
SpotPortfolio {
    engine_id: 37fd5e92-a909-4638-a539-f40262c761a7,
    exchanges: [
        BinanceSpot,
        PoloniexSpot,
    ],
    repository: SpotInMemoryRepository {
        open_positions: {
            ExchangeAssetId(
                "BINANCESPOT_OPUSDT",
            ): Position2 {
                order_request_time: 2024-12-22T02:55:32.950258Z,
                last_execution_time: 2024-12-22T02:55:50.400Z,
                position_id: ExchangeAssetId(
                    "BINANCESPOT_OPUSDT",
                ),
                exchange: BinanceSpot,
                asset: AssetFormatted(
                    "OPUSDT",
                ),
                side: Buy,
                quantity: 4.0,
                fees: 0.004,
                current_symbol_price: 1.7460927367897454,
                current_value_gross: 0.0,
                filled_gross: 6.984,
                enter_avg_price: 1.746,
                enter_value_gross: 0.0,
                unrealised_profit_loss: -0.004,
                realised_profit_loss: 0.0,
            },
        },
        closed_positions: {},
        current_balance: {
            SpotBalanceId(
                "binancespot_usdt",
            ): Balance {
                total: 2.40736955,
                available: 0.0,
            },
            SpotBalanceId(
                "binancespot_arb",
            ): Balance {
                total: 0.0894,
                available: 0.0,
            },
            SpotBalanceId(
                "poloniexspot_usdt",
            ): Balance {
                total: 11.06022807796,
                available: 0.0,
            },
            SpotBalanceId(
                "poloniexspot_op",
            ): Balance {
                total: 8.04e-5,
                available: 0.0,
            },
            SpotBalanceId(
                "binancespot_bnb",
            ): Balance {
                total: 0.0,
                available: 0.0,
            },
            SpotBalanceId(
                "binancespot_op",
            ): Balance {
                total: 3.99611,
                available: 0.0,
            },
            SpotBalanceId(
                "binancespot_usdce",
            ): Balance {
                total: 1.0,
                available: 0.0,
            },
        },
    },
    allocator: SpotArbAllocator,
}
####### ORDER UPDATED ##########
OrderEvent {
    order_request_time: 2024-12-22T02:55:32.950258Z,
    exchange: BinanceSpot,
    client_order_id: Some(
        ClientOrderId(
            "T01L6y9R4FvUKe15X8Si4n",
        ),
    ),
    instrument: Instrument {
        base: "op",
        quote: "usdt",
    },
    market_meta: MarketMeta {
        close: 1.746,
        time: 2024-12-22T02:55:32.950255Z,
    },
    decision: Long,
    original_quantity: 4.0,
    cumulative_quantity: 4.0,
    order_kind: Limit,
    exchange_order_status: Some(
        Filled,
    ),
    internal_order_state: Open,
    filled_gross: 6.984,
    enter_avg_price: 1.746,
    fees: 0.004,
    last_execution_time: Some(
        2024-12-22T02:55:50.400Z,
    ),
}
AccountDataBalance: AccountDataBalance {
    asset: "OP",
    exchange: BinanceSpot,
    balance: Balance {
        total: 3.99611,
        available: 0.0,
    },
}

########################################################################
########################################################################

### Poloniex ###
arb portfolio: Mutex {
    data: SpotPortfolio {
        engine_id: a5ee10a3-198d-4aed-a8b2-c7cd592a5028,
        exchanges: [
            BinanceSpot,
            PoloniexSpot,
        ],
        repository: SpotInMemoryRepository {
            open_positions: {},
            closed_positions: {},
            current_balance: {
                SpotBalanceId(
                    "poloniexspot_usdt",
                ): Balance {
                    total: 11.06022807796,
                    available: 0.0,
                },
                SpotBalanceId(
                    "poloniexspot_op",
                ): Balance {
                    total: 8.04e-5,
                    available: 0.0,
                },
                SpotBalanceId(
                    "binancespot_arb",
                ): Balance {
                    total: 0.0894,
                    available: 0.0,
                },
                SpotBalanceId(
                    "binancespot_usdce",
                ): Balance {
                    total: 1.0,
                    available: 0.0,
                },
                SpotBalanceId(
                    "binancespot_op",
                ): Balance {
                    total: 3.99611,
                    available: 0.0,
                },
                SpotBalanceId(
                    "binancespot_usdt",
                ): Balance {
                    total: 2.40736955,
                    available: 0.0,
                },
            },
        },
        allocator: SpotArbAllocator,
    },
}
AccountDataOrder: AccountDataOrder {
    exchange: PoloniexSpot,
    client_order_id: "393725213122150400",
    asset: "OP_USDT",
    price: 0.0,
    quantity: 0.0,
    status: New,
    execution_time: 2024-12-22T03:23:25.869Z,
    side: Buy,
    fee: 0.0,
    filled_gross: 0.0,
}
####### ORDER BEFORE UPDATE ##########
OrderEvent {
    order_request_time: 2024-12-22T03:23:25.526579Z,
    exchange: PoloniexSpot,
    client_order_id: None,
    instrument: Instrument {
        base: "op",
        quote: "usdt",
    },
    market_meta: MarketMeta {
        close: 1.759,
        time: 2024-12-22T03:23:25.526569Z,
    },
    decision: Long,
    original_quantity: 3.0,
    cumulative_quantity: 0.0,
    order_kind: Limit,
    exchange_order_status: None,
    internal_order_state: InTransit,
    filled_gross: 0.0,
    enter_avg_price: 0.0,
    fees: 0.0,
    last_execution_time: None,
}
############ IN NONE ############
############ UPDATE FROM FILL 2 ############
SpotPortfolio {
    engine_id: a5ee10a3-198d-4aed-a8b2-c7cd592a5028,
    exchanges: [
        BinanceSpot,
        PoloniexSpot,
    ],
    repository: SpotInMemoryRepository {
        open_positions: {
            ExchangeAssetId(
                "POLONIEXSPOT_OP_USDT",
            ): Position2 {
                order_request_time: 2024-12-22T03:23:25.526579Z,
                last_execution_time: 2024-12-22T03:23:25.869Z,
                position_id: ExchangeAssetId(
                    "POLONIEXSPOT_OP_USDT",
                ),
                exchange: PoloniexSpot,
                asset: AssetFormatted(
                    "OP_USDT",
                ),
                side: Buy,
                quantity: 0.0,
                fees: 0.0,
                current_symbol_price: 0.0,
                current_value_gross: 0.0,
                filled_gross: 0.0,
                enter_avg_price: 0.0,
                enter_value_gross: 0.0,
                unrealised_profit_loss: -0.0,
                realised_profit_loss: 0.0,
            },
        },
        closed_positions: {},
        current_balance: {
            SpotBalanceId(
                "poloniexspot_usdt",
            ): Balance {
                total: 5.78322807796,
                available: 0.0,
            },
            SpotBalanceId(
                "poloniexspot_op",
            ): Balance {
                total: 8.04e-5,
                available: 0.0,
            },
            SpotBalanceId(
                "binancespot_arb",
            ): Balance {
                total: 0.0894,
                available: 0.0,
            },
            SpotBalanceId(
                "binancespot_usdce",
            ): Balance {
                total: 1.0,
                available: 0.0,
            },
            SpotBalanceId(
                "binancespot_op",
            ): Balance {
                total: 3.99611,
                available: 0.0,
            },
            SpotBalanceId(
                "binancespot_usdt",
            ): Balance {
                total: 2.40736955,
                available: 0.0,
            },
        },
    },
    allocator: SpotArbAllocator,
}
####### ORDER UPDATED ##########
OrderEvent {
    order_request_time: 2024-12-22T03:23:25.526579Z,
    exchange: PoloniexSpot,
    client_order_id: Some(
        ClientOrderId(
            "393725213122150400",
        ),
    ),
    instrument: Instrument {
        base: "op",
        quote: "usdt",
    },
    market_meta: MarketMeta {
        close: 1.759,
        time: 2024-12-22T03:23:25.526569Z,
    },
    decision: Long,
    original_quantity: 3.0,
    cumulative_quantity: 0.0,
    order_kind: Limit,
    exchange_order_status: Some(
        New,
    ),
    internal_order_state: Open,
    filled_gross: 0.0,
    enter_avg_price: NaN,
    fees: 0.0,
    last_execution_time: Some(
        2024-12-22T03:23:25.869Z,
    ),
}
AccountDataBalance: AccountDataBalance {
    asset: "OP",
    exchange: PoloniexSpot,
    balance: Balance {
        total: 2.0000804,
        available: 0.0,
    },
}
AccountDataBalance: AccountDataBalance {
    asset: "OP",
    exchange: PoloniexSpot,
    balance: Balance {
        total: 1.9960804,
        available: 0.0,
    },
}
AccountDataOrder: AccountDataOrder {
    exchange: PoloniexSpot,
    client_order_id: "393725213122150400",
    asset: "OP_USDT",
    price: 1.759,
    quantity: 2.0,
    status: PartiallyFilled,
    execution_time: 2024-12-22T03:23:25.869Z,
    side: Buy,
    fee: 0.004,
    filled_gross: 3.518,
}
####### ORDER BEFORE UPDATE ##########
OrderEvent {
    order_request_time: 2024-12-22T03:23:25.526579Z,
    exchange: PoloniexSpot,
    client_order_id: Some(
        ClientOrderId(
            "393725213122150400",
        ),
    ),
    instrument: Instrument {
        base: "op",
        quote: "usdt",
    },
    market_meta: MarketMeta {
        close: 1.759,
        time: 2024-12-22T03:23:25.526569Z,
    },
    decision: Long,
    original_quantity: 3.0,
    cumulative_quantity: 0.0,
    order_kind: Limit,
    exchange_order_status: Some(
        New,
    ),
    internal_order_state: Open,
    filled_gross: 0.0,
    enter_avg_price: NaN,
    fees: 0.0,
    last_execution_time: Some(
        2024-12-22T03:23:25.869Z,
    ),
}
############ IN SOME ############
############ UPDATE FROM FILL 2 ############
SpotPortfolio {
    engine_id: a5ee10a3-198d-4aed-a8b2-c7cd592a5028,
    exchanges: [
        BinanceSpot,
        PoloniexSpot,
    ],
    repository: SpotInMemoryRepository {
        open_positions: {
            ExchangeAssetId(
                "POLONIEXSPOT_OP_USDT",
            ): Position2 {
                order_request_time: 2024-12-22T03:23:25.526579Z,
                last_execution_time: 2024-12-22T03:23:25.869Z,
                position_id: ExchangeAssetId(
                    "POLONIEXSPOT_OP_USDT",
                ),
                exchange: PoloniexSpot,
                asset: AssetFormatted(
                    "OP_USDT",
                ),
                side: Buy,
                quantity: 2.0,
                fees: 0.004,
                current_symbol_price: 1.7614,
                current_value_gross: 0.0,
                filled_gross: 3.518,
                enter_avg_price: 1.759,
                enter_value_gross: 0.0,
                unrealised_profit_loss: -0.004,
                realised_profit_loss: 0.0,
            },
        },
        closed_positions: {},
        current_balance: {
            SpotBalanceId(
                "poloniexspot_usdt",
            ): Balance {
                total: 5.78322807796,
                available: 0.0,
            },
            SpotBalanceId(
                "poloniexspot_op",
            ): Balance {
                total: 1.9960804,
                available: 0.0,
            },
            SpotBalanceId(
                "binancespot_arb",
            ): Balance {
                total: 0.0894,
                available: 0.0,
            },
            SpotBalanceId(
                "binancespot_usdce",
            ): Balance {
                total: 1.0,
                available: 0.0,
            },
            SpotBalanceId(
                "binancespot_op",
            ): Balance {
                total: 3.99611,
                available: 0.0,
            },
            SpotBalanceId(
                "binancespot_usdt",
            ): Balance {
                total: 2.40736955,
                available: 0.0,
            },
        },
    },
    allocator: SpotArbAllocator,
}
####### ORDER UPDATED ##########
OrderEvent {
    order_request_time: 2024-12-22T03:23:25.526579Z,
    exchange: PoloniexSpot,
    client_order_id: Some(
        ClientOrderId(
            "393725213122150400",
        ),
    ),
    instrument: Instrument {
        base: "op",
        quote: "usdt",
    },
    market_meta: MarketMeta {
        close: 1.759,
        time: 2024-12-22T03:23:25.526569Z,
    },
    decision: Long,
    original_quantity: 3.0,
    cumulative_quantity: 2.0,
    order_kind: Limit,
    exchange_order_status: Some(
        PartiallyFilled,
    ),
    internal_order_state: Open,
    filled_gross: 3.518,
    enter_avg_price: 1.759,
    fees: 0.004,
    last_execution_time: Some(
        2024-12-22T03:23:25.869Z,
    ),
}
AccountDataBalance: AccountDataBalance {
    asset: "OP",
    exchange: PoloniexSpot,
    balance: Balance {
        total: 2.9960804,
        available: 0.0,
    },
}
AccountDataBalance: AccountDataBalance {
    asset: "OP",
    exchange: PoloniexSpot,
    balance: Balance {
        total: 2.9940804,
        available: 0.0,
    },
}
AccountDataOrder: AccountDataOrder {
    exchange: PoloniexSpot,
    client_order_id: "393725213122150400",
    asset: "OP_USDT",
    price: 1.759,
    quantity: 1.0,
    status: Filled,
    execution_time: 2024-12-22T03:23:25.869Z,
    side: Buy,
    fee: 0.002,
    filled_gross: 5.277,
}
####### ORDER BEFORE UPDATE ##########
OrderEvent {
    order_request_time: 2024-12-22T03:23:25.526579Z,
    exchange: PoloniexSpot,
    client_order_id: Some(
        ClientOrderId(
            "393725213122150400",
        ),
    ),
    instrument: Instrument {
        base: "op",
        quote: "usdt",
    },
    market_meta: MarketMeta {
        close: 1.759,
        time: 2024-12-22T03:23:25.526569Z,
    },
    decision: Long,
    original_quantity: 3.0,
    cumulative_quantity: 2.0,
    order_kind: Limit,
    exchange_order_status: Some(
        PartiallyFilled,
    ),
    internal_order_state: Open,
    filled_gross: 3.518,
    enter_avg_price: 1.759,
    fees: 0.004,
    last_execution_time: Some(
        2024-12-22T03:23:25.869Z,
    ),
}
############ IN SOME ############
############ UPDATE FROM FILL 2 ############
SpotPortfolio {
    engine_id: a5ee10a3-198d-4aed-a8b2-c7cd592a5028,
    exchanges: [
        BinanceSpot,
        PoloniexSpot,
    ],
    repository: SpotInMemoryRepository {
        open_positions: {
            ExchangeAssetId(
                "POLONIEXSPOT_OP_USDT",
            ): Position2 {
                order_request_time: 2024-12-22T03:23:25.526579Z,
                last_execution_time: 2024-12-22T03:23:25.869Z,
                position_id: ExchangeAssetId(
                    "POLONIEXSPOT_OP_USDT",
                ),
                exchange: PoloniexSpot,
                asset: AssetFormatted(
                    "OP_USDT",
                ),
                side: Buy,
                quantity: 3.0,
                fees: 0.006,
                current_symbol_price: 1.7541214923126545,
                current_value_gross: 3.508242984625309,
                filled_gross: 5.277,
                enter_avg_price: 1.7590000000000001,
                enter_value_gross: 0.0,
                unrealised_profit_loss: 3.502242984625309,
                realised_profit_loss: 0.0,
            },
        },
        closed_positions: {},
        current_balance: {
            SpotBalanceId(
                "poloniexspot_usdt",
            ): Balance {
                total: 5.78322807796,
                available: 0.0,
            },
            SpotBalanceId(
                "poloniexspot_op",
            ): Balance {
                total: 2.9940804,
                available: 0.0,
            },
            SpotBalanceId(
                "binancespot_arb",
            ): Balance {
                total: 0.0894,
                available: 0.0,
            },
            SpotBalanceId(
                "binancespot_usdce",
            ): Balance {
                total: 1.0,
                available: 0.0,
            },
            SpotBalanceId(
                "binancespot_op",
            ): Balance {
                total: 3.99611,
                available: 0.0,
            },
            SpotBalanceId(
                "binancespot_usdt",
            ): Balance {
                total: 2.40736955,
                available: 0.0,
            },
        },
    },
    allocator: SpotArbAllocator,
}
####### ORDER UPDATED ##########
OrderEvent {
    order_request_time: 2024-12-22T03:23:25.526579Z,
    exchange: PoloniexSpot,
    client_order_id: Some(
        ClientOrderId(
            "393725213122150400",
        ),
    ),
    instrument: Instrument {
        base: "op",
        quote: "usdt",
    },
    market_meta: MarketMeta {
        close: 1.759,
        time: 2024-12-22T03:23:25.526569Z,
    },
    decision: Long,
    original_quantity: 3.0,
    cumulative_quantity: 3.0,
    order_kind: Limit,
    exchange_order_status: Some(
        Filled,
    ),
    internal_order_state: Open,
    filled_gross: 5.277,
    enter_avg_price: 1.7590000000000001,
    fees: 0.006,
    last_execution_time: Some(
        2024-12-22T03:23:25.869Z,
    ),
}
*/
