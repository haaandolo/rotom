use chrono::Utc;
use futures::StreamExt;
use parking_lot::Mutex;
use rotom_data::{
    event_models::market_event::{DataKind, MarketEvent},
    protocols::ws::ws_parser::{StreamParser, WebSocketParser},
    shared::subscription_models::{ExchangeId, Instrument, StreamKind},
    streams::builder::dynamic,
    Market, MarketFeed, MarketMeta,
};
use rotom_main::{
    engine::Engine,
    trader::{arb_trader::ArbTrader, single_trader::SingleMarketTrader},
};
use rotom_oms::{
    event::{Event, EventTx},
    execution::{
        exchange_client::{
            binance::{binance_client::BinanceExecution, requests::responses::BinanceResponses},
            poloniex::poloniex_testing,
        },
        simulated::{Config, SimulatedExecution},
        ExecutionClient2, Fees,
    },
    portfolio::{
        allocator::DefaultAllocator, portfolio::MetaPortfolio,
        repository::in_memory::InMemoryRepository, risk::DefaultRisk, OrderEvent, OrderType,
    },
    statistic::summary::{
        trading::{Config as StatisticConfig, TradingSummary},
        Initialiser,
    },
};
use rotom_strategy::{spread::SpreadStategy, Decision};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::mpsc::{self, UnboundedReceiver};
use uuid::Uuid;

const ENGINE_RUN_TIMEOUT: Duration = Duration::from_secs(5000);

/*
Some(
    Ok(
        NewOrderResult(
            BinanceNewOrderResponseResult {
                id: "fe0d5e1b-df4d-42d1-aa3b-33676e403938",
                status: 200,
                result: BinanceNewOrderResponseResultData {
                    symbol: "OPUSDT",
                    order_id: 1581077079,
                    order_list_id: -1,
                    client_order_id: "O5eBTHcWHo2vFtrmB9rE7k", <-- THIS
                    transact_time: 1725768574940,
                    price: 1.436,
                    orig_qty: 5.0,
                    executed_qty: 0.0,
                    cummulative_quote_qty: 0.0,
                    status: New,
                    time_in_force: GTC,
                    type: "LIMIT",
                    side: "BUY",
                    working_time: 1725768574940,
                    fills: [],
                    self_trade_prevention_mode: "EXPIRE_MAKER",
                },
                rate_limits: [
                    RateLimit {
                        rate_limit_type: "ORDERS",
                        interval: "SECOND",
                        interval_num: 10,
                        limit: 100,
                        count: 1,
                    },
                    RateLimit {
                        rate_limit_type: "ORDERS",
                        interval: "DAY",
                        interval_num: 1,
                        limit: 200000,
                        count: 2,
                    },
                    RateLimit {
                        rate_limit_type: "REQUEST_WEIGHT",
                        interval: "MINUTE",
                        interval_num: 1,
                        limit: 6000,
                        count: 3,
                    },
                ],
            },
        ),
    ),
)
*/
/*----- */
// Main
/*----- */
#[tokio::main]
pub async fn main() {
    // Initialise logging
    init_logging();

    /*----- */
    // Testing
    /*----- */
    // Order
    let mut order = OrderEvent {
        time: Utc::now(),
        exchange: ExchangeId::BinanceSpot,
        instrument: Instrument::new("op", "usdt"),
        market_meta: MarketMeta {
            time: Utc::now(),
            close: 1.420,
        },
        decision: Decision::Long,
        quantity: 5.0,
        order_type: OrderType::Limit,
    };

    // Test Binance Execution
    let binance_exe = BinanceExecution::init().await.unwrap();
    // binance_exe.open_order(order.clone()).await;
    // order.market_meta.close = 1.421;
    // binance_exe.open_order(order).await;
    // binance_exe.cancel_order("RCRGTqNC0cCb9zv64aUxtS".to_string()).await;
    binance_exe.cancel_order_all("OPUSDT".to_string()).await;
    binance_exe.receive_reponses().await;

    // let _ = poloniex_testing().await;

    /*----- */
    // Trader builder
    /*----- */
    // // Testing
    // let res = reqwest::get("https://api.binance.us/api/v3/depth?symbol=LTCBTC")
    //     .await
    //     .unwrap()
    //     .text()
    //     .await
    //     .unwrap();

    // println!("{:#?}", res);

    // Engine id
    let engine_id = Uuid::new_v4();

    // Market
    let markets = vec![
        Market::new(ExchangeId::BinanceSpot, Instrument::new("op", "usdt")),
        Market::new(ExchangeId::PoloniexSpot, Instrument::new("op", "usdt")),
    ];

    // Channels
    // Create channel to distribute Commands to the Engine & it's Traders (eg/ Command::Terminate)
    let (_command_tx, command_rx) = mpsc::channel(20);

    // Create channel for each Trader so the Engine can distribute Commands to it
    let (trader_command_tx, trader_command_rx) = mpsc::channel(10);

    // Create Event channel to listen to all Engine Events in real-time
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let event_tx = EventTx::new(event_tx);

    // Portfolio
    let portfolio = Arc::new(Mutex::new(
        MetaPortfolio::builder()
            .engine_id(engine_id)
            .markets(markets.clone())
            .starting_cash(10000.0)
            .repository(InMemoryRepository::<TradingSummary>::new())
            .allocation_manager(DefaultAllocator {
                default_order_value: 100.0,
            })
            .risk_manager(DefaultRisk {})
            .statistic_config(StatisticConfig {
                starting_equity: 10_000.0,
                trading_days_per_year: 365,
                risk_free_return: 0.0,
            })
            .build_init()
            .unwrap(),
    ));

    // Build traders
    // let single_market_trader = SingleMarketTrader::builder()
    //     .engine_id(engine_id)
    //     .market(markets[0].clone())
    //     .command_rx(trader_command_rx)
    //     .event_tx(event_tx.clone())
    //     .portfolio(Arc::clone(&portfolio))
    //     .data(live::MarketFeed::new(stream_trades().await))
    //     .strategy(SpreadStategy::new())
    //     .execution(SimulatedExecution::new(Config {
    //         simulated_fees_pct: Fees {
    //             exchange: 0.01,
    //             slippage: 0.05,
    //             network: 0.0,
    //         },
    //     }))
    //     .build()
    //     .unwrap();
    // let single_traders = vec![single_market_trader];

    let arb_trader = ArbTrader::builder()
        .engine_id(engine_id)
        .market(markets.clone())
        .command_rx(trader_command_rx)
        .event_tx(event_tx)
        .portfolio(Arc::clone(&portfolio))
        .data(MarketFeed::new(stream_trades().await))
        .strategy(SpreadStategy::new())
        .execution(SimulatedExecution::new(Config {
            simulated_fees_pct: Fees {
                exchange: 0.01,
                slippage: 0.05,
                network: 0.0,
            },
        }))
        .build()
        .unwrap();
    let arb_traders = vec![arb_trader];

    // Build engine TODO: (check the commands are doing what it is supposed to)
    let trader_command_txs = markets
        .into_iter()
        .map(|market| (market, trader_command_tx.clone()))
        .collect::<HashMap<_, _>>();

    let engine = Engine::builder()
        .engine_id(engine_id)
        .command_rx(command_rx)
        .portfolio(portfolio)
        .traders(arb_traders)
        .trader_command_txs(trader_command_txs)
        .statistics_summary(TradingSummary::init(StatisticConfig {
            starting_equity: 1000.0,
            trading_days_per_year: 365,
            risk_free_return: 0.0,
        }))
        .build()
        .expect("failed to build engine");

    // Run Engine trading & listen to Events it produces
    tokio::spawn(listen_to_engine_events(event_rx));

    let _ = tokio::time::timeout(ENGINE_RUN_TIMEOUT, engine.run()).await;
}

/*----- */
// Setup data feed
/*----- */
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

/*----- */
// Event listener
/*----- */
// Listen to Events that occur in the Engine. These can be used for updating event-sourcing,
// updating dashboard, etc etc.
async fn listen_to_engine_events(mut event_rx: mpsc::UnboundedReceiver<Event>) {
    while let Some(event) = event_rx.recv().await {
        match event {
            Event::Market(market) => {
                // Market Event occurred in Engine
                // println!("{market:?}");
            }
            Event::Signal(signal) => {
                // Signal Event occurred in Engine
                // println!("{signal:?}");
            }
            Event::SignalForceExit(_) => {
                // SignalForceExit Event occurred in Engine
            }
            Event::OrderNew(new_order) => {
                // OrderNew Event occurred in Engine
                // println!("{new_order:?}");
            }
            Event::OrderUpdate => {
                // OrderUpdate Event occurred in Engine
            }
            Event::Fill(fill_event) => {
                // Fill Event occurred in Engine
                // println!("{fill_event:?}");
            }
            Event::PositionNew(new_position) => {
                // PositionNew Event occurred in Engine
                // println!("{new_position:?}");
            }
            Event::PositionUpdate(updated_position) => {
                // PositionUpdate Event occurred in Engine
                // println!("{updated_position:?}");
            }
            Event::PositionExit(exited_position) => {
                // PositionExit Event occurred in Engine
                // println!("{exited_position:?}");
            }
            Event::Balance(balance_update) => {
                // Balance update Event occurred in Engine
                // println!("{balance_update:?}");
            }
        }
    }
}

/*----- */
// Logging config
/*----- */
fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::filter::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        // Disable colours on release builds
        .with_ansi(cfg!(debug_assertions))
        // Enable Json formatting
        .json()
        // Install this Tracing subscriber as global default
        .init()
}

/*----- */
// Todo
/*----- */
// - change level size to quantity (name change)
// - change r#type to enum instead of string
