use rotom_data::shared::subscription_models::Instrument;
use rotom_data::MarketFeed;
use rotom_data::{
    model::market_event::{DataKind, MarketEvent},
    shared::subscription_models::ExchangeId,
    Feed, MarketGenerator,
};
use rotom_oms::execution_manager::builder::TraderId;
use rotom_oms::{
    event::Event,
    model::{
        account_data::AccountData,
        order::{ExecutionRequest, OrderEvent},
    },
};
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
    liquid_exchange: mpsc::UnboundedSender<ExecutionRequest>,
    illiquid_exchange: mpsc::UnboundedSender<ExecutionRequest>,
    order_generator: SpotArbOrderGenerator,
    order_update_rx: mpsc::Receiver<AccountData>,
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
                    // markets = &*format!("{:?}", self.markets), // todo
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
                        // market = ?self.markets, // todo
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
                        println!("#################");
                        println!("Market data");
                        println!("#################");
                        println!("{:#?}", market_event);

                        // Process latest market event and generate order if applicable
                        if let Some(mut new_order) =
                            self.order_generator.process_market_event(&market_event)
                        {
                            self.event_queue.push_back(Event::OrderEvaluate(new_order));
                        }
                    }
                    Event::OrderNew(new_order) => {
                        match &self.meta_data.order {
                            Some(_) => {
                                // self.process_existing_order(new_order).await; // uncomment to start limit order
                            }
                            None => {
                                // self.process_new_order().await; // uncomment to start limit order
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        debug!(
            engine_id = &*self.engine_id.to_string(),
            // market = &*format!("{:?}", self.markets), // todo
            message = "Trader trading loop stopped"
        )
    }
}

/*----- */
// Single Market Trader builder
/*----- */
#[derive(Debug, Default)]
pub struct SpotArbTraderBuilder {
    pub engine_id: Option<Uuid>,
    pub trader_id: Option<TraderId>,
    pub command_rx: Option<mpsc::Receiver<Command>>,
    pub data: Option<MarketFeed<MarketEvent<DataKind>>>,
    pub liquid_exchange: Option<mpsc::UnboundedSender<ExecutionRequest>>,
    pub illiquid_exchange: Option<mpsc::UnboundedSender<ExecutionRequest>>,
    pub order_generator: Option<SpotArbOrderGenerator>,
    pub send_order_tx: Option<mpsc::UnboundedSender<ExecutionRequest>>,
    pub order_update_rx: Option<mpsc::Receiver<AccountData>>,
    pub meta_data: Option<SpotArbTraderMetaData>,
}

impl SpotArbTraderBuilder {
    pub fn new() -> Self {
        Self {
            engine_id: None,
            trader_id: None,
            command_rx: None,
            data: None,
            liquid_exchange: None,
            illiquid_exchange: None,
            order_generator: None,
            send_order_tx: None,
            order_update_rx: None,
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

    pub fn liquid_exchange(self, value: mpsc::UnboundedSender<ExecutionRequest>) -> Self {
        Self {
            liquid_exchange: Some(value),
            ..self
        }
    }

    pub fn illiquid_exchange(self, value: mpsc::UnboundedSender<ExecutionRequest>) -> Self {
        Self {
            illiquid_exchange: Some(value),
            ..self
        }
    }

    pub fn order_generator(self, value: SpotArbOrderGenerator) -> Self {
        Self {
            order_generator: Some(value),
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
            liquid_exchange: self
                .liquid_exchange
                .ok_or(EngineError::BuilderIncomplete("liquid_exchange"))?,
            illiquid_exchange: self
                .illiquid_exchange
                .ok_or(EngineError::BuilderIncomplete("illiquid_exchange"))?,
            order_generator: self
                .order_generator
                .ok_or(EngineError::BuilderIncomplete("order_generator"))?,
            order_update_rx: self
                .order_update_rx
                .ok_or(EngineError::BuilderIncomplete("order_update_rx"))?,
            event_queue: VecDeque::with_capacity(2),
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


#################3#####

### Raw Acc data ###
Balance(
    AccountDataBalance {
        asset: "USDT",
        exchange: PoloniexSpot,
        balance: Balance {
            total: 4.74845647748,
            available: 0.0,
        },
    },
)
### Raw Acc data ###
Order(
    AccountDataOrder {
        exchange: PoloniexSpot,
        client_order_id: "399211577842257920",
        asset: "OP_USDT",
        price: 0.0,
        quantity: 0.0,
        status: New,
        execution_time: 2025-01-06T06:44:17.150Z,
        side: Buy,
        fee: 0.0,
        filled_gross: 0.0,
    },
)
### Raw Acc data ###
Balance(
    AccountDataBalance {
        asset: "USDT",
        exchange: PoloniexSpot,
        balance: Balance {
            total: 4.74845647748,
            available: 0.0,
        },
    },
)
### Raw Acc data ###
Balance(
    AccountDataBalance {
        asset: "USDT",
        exchange: PoloniexSpot,
        balance: Balance {
            total: 4.74856948748,
            available: 0.0,
        },
    },
)
### Raw Acc data ###
Balance(
    AccountDataBalance {
        asset: "OP",
        exchange: PoloniexSpot,
        balance: Balance {
            total: 1.8929852,
            available: 0.0,
        },
    },
)
### Raw Acc data ###
Balance(
    AccountDataBalance {
        asset: "OP",
        exchange: PoloniexSpot,
        balance: Balance {
            total: 1.8891994,
            available: 0.0,
        },
    },
)
### Raw Acc data ###
Order(
    AccountDataOrder {
        exchange: PoloniexSpot,
        client_order_id: "399211577842257920",
        asset: "OP_USDT",
        price: 2.1131,
        quantity: 1.8929,
        status: Filled,
        execution_time: 2025-01-06T06:44:17.150Z,
        side: Buy,
        fee: 0.0037858,
        filled_gross: 3.99988699,
    },
)
AccountDataOrder: AccountDataOrder {
    exchange: PoloniexSpot,
    client_order_id: "399211577842257920",
    asset: "OP_USDT",
    price: 0.0,
    quantity: 0.0,
    status: New,
    execution_time: 2025-01-06T06:44:17.150Z,
    side: Buy,
    fee: 0.0,
    filled_gross: 0.0,
}
OrderEvent {
    order_request_time: 2025-01-06T06:44:08.114939Z,
    exchange: PoloniexSpot,
    client_order_id: Some(
        ClientOrderId(
            "399211577842257920",
        ),
    ),
    instrument: Instrument {
        base: "op",
        quote: "usdt",
    },
    market_meta: MarketMeta {
        close: 2.061626,
        time: 2025-01-06T06:44:08.114940Z,
    },
    decision: Long,
    original_quantity: 1.0,
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
        2025-01-06T06:44:17.150Z,
    ),
}
AccountDataOrder: AccountDataOrder {
    exchange: PoloniexSpot,
    client_order_id: "399211577842257920",
    asset: "OP_USDT",
    price: 2.1131,
    quantity: 1.8929,
    status: Filled,
    execution_time: 2025-01-06T06:44:17.150Z,
    side: Buy,
    fee: 0.0037858,
    filled_gross: 3.99988699,
}
OrderEvent {
    order_request_time: 2025-01-06T06:44:08.114939Z,
    exchange: PoloniexSpot,
    client_order_id: Some(
        ClientOrderId(
            "399211577842257920",
        ),
    ),
    instrument: Instrument {
        base: "op",
        quote: "usdt",
    },
    market_meta: MarketMeta {
        close: 2.061626,
        time: 2025-01-06T06:44:08.114940Z,
    },
    decision: Long,
    original_quantity: 1.0,
    cumulative_quantity: 1.8929,
    order_kind: Limit,
    exchange_order_status: Some(
        Filled,
    ),
    internal_order_state: Open,
    filled_gross: 3.99988699,
    enter_avg_price: 2.1130999999999998,
    fees: 0.0037858,
    last_execution_time: Some(
        2025-01-06T06:44:17.150Z,
    ),
}
*/
