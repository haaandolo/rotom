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
    engine::{self, Engine},
    trader::{arb_trader::ArbTrader, single_trader::SingleMarketTrader},
};
use rotom_oms::{
    event::{Event, EventTx},
    exchange::{
        binance::binance_client::BinanceExecution, poloniex::poloniex_client::PoloniexExecution,
        ExecutionClient2,
    },
    execution::{
        arena::spot_arb::spot_arb_executor::SpotArbExecutor,
        simulated::{Config, SimulatedExecution},
        Fees,
    },
    model::{
        order::{Order, RequestOpen},
        ClientOrderId,
    },
    portfolio::{
        allocator::{default_allocator::DefaultAllocator, spot_arb_allocator::SpotArbAllocator},
        persistence::{in_memory::InMemoryRepository, in_memory2::InMemoryRepository2},
        portfolio_type::{default_portfolio::MetaPortfolio, spot_portfolio::SpotPortfolio},
        risk_manager::default_risk_manager::DefaultRisk,
    },
    statistic::summary::{
        trading::{Config as StatisticConfig, TradingSummary},
        Initialiser,
    },
};
use rotom_strategy::{spread::SpreadStategy, Decision};
use std::{
    collections::HashMap,
    sync::{atomic, Arc},
    time::Duration,
};
use tokio::{
    sync::mpsc::{self, UnboundedReceiver},
    time::sleep,
};
use uuid::Uuid;

const ENGINE_RUN_TIMEOUT: Duration = Duration::from_secs(5000);

// client_order_id: "O5eBTHcWHo2vFtrmB9rE7k", <-- THIS for cancel an order

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

    ////////////////////////////////////////////////////
    // Order
    let mut order = Order {
        exchange: ExchangeId::BinanceSpot,
        instrument: Instrument::new("op", "usdt"),
        client_order_id: ClientOrderId(Uuid::new_v4()),
        side: rotom_oms::model::Side::Buy,
        state: RequestOpen {
            kind: rotom_oms::model::OrderKind::Limit,
            price: 1.0,
            quantity: 5.0,
            decision: Decision::Long,
        },
    };

    // Test Binance Execution
    // let binance_exe = BinanceExecution::create_http_client().unwrap();
    // let res = binance_exe.get_balance_all().await;
    // let res: Vec<AssetBalance> = res.unwrap().into();
    // let res = binance_exe
    //     .wallet_transfer(
    //         "USDT".to_string(),
    //         "TBw5BWoS97tWrVr7PSuBtUQeBXU6eJZpyg".to_string(),
    //         Some("TRX".to_string()),
    //         10.0,
    //     )
    //     .await;
    // let res = binance_exe.open_order(order.clone()).await;
    // let res = binance_exe
    //     .cancel_order("YTirLsT3mwmdxDED6HBH5c".to_string(), "OPUSDT".to_string())
    //     .await;
    // binance_exe.cancel_order_all("OPUSDT".to_string()).await;
    // println!("{:#?}", res);
    // binance_exe.receive_responses().await;

    ////////////////////////////////////////////////////
    // Test Poloniex Execution
    // let polo_exe = PoloniexExecution::create_http_client().unwrap();
    // let res = polo_exe.open_order(order.clone()).await;

    // order.market_meta.close = 0.90;
    // let res = polo_exe.open_order(order.clone()).await;

    // let res = polo_exe
    //     .cancel_order(
    //         "0adc007d-836f-424c-b954-b05e297269c8".to_string(),
    //         "None".to_string(),
    //     )
    //     .await;

    // let res= polo_exe.cancel_order_all("OP_USDT".to_string()).await;

    // polo_exe.receive_responses().await;

    // let res = polo_exe.wallet_transfer(
    //     "USDT".to_string(),
    //     "TLHWcKwg5gdTXsv6Bko9srkiKZomRBYCr2".to_string(),
    //     Some("TRX".to_string()),
    //     5.0
    // ).await;
    // println!("---> {:#?}", res);

    ////////////////////////////////////////////////
    //>>> UNCOMMENT FORM HERE ALL THE WAY DOWN <<<
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
    /////////////////////////////////////////////////
    // Arena
    let mut arb_exe = SpotArbExecutor::<BinanceExecution, PoloniexExecution>::init()
        .await
        .unwrap();

    while let Some(msg) = arb_exe.streams.recv().await {
        println!("{:#?}", msg);
    }

    // let arb_exe = ArbExecutor::<BinanceExecution, PoloniexExecution>::default();

    // let mut polo_exe = PoloniexExecution::init().await.unwrap();
    // while let Some(message) = polo_exe.rx.recv().await {
    //     println!("--->>> {:#?}", message);
    // }

    // let mut bin_exe = BinanceExecution::init().await.unwrap();
    // while let Some(message) = bin_exe.rx.recv().await {
    //     println!("--->>> {:#?}", message);
    // }

    /////////////////////////////////////////////////
    // Engine id
    let engine_id = Uuid::new_v4();

    ////////////////////////////////////////////////////
    // Portfolio
    let arb_portfolio = Arc::new(Mutex::new(
        SpotPortfolio::new(
            engine_id,
            vec![ExchangeId::BinanceSpot, ExchangeId::PoloniexSpot],
            InMemoryRepository2::default(),
            SpotArbAllocator,
        )
        .init()
        .await
        .unwrap(),
    ));

    println!("{:#?}", arb_portfolio);
    ///////////////////////////////////////////////////
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
        .portfolio(Arc::clone(&arb_portfolio))
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
// - impl From trait for order and balance for polo and bin
// - figure out the balance +ve and -ve of quote and base asset for portfolio when the fill is updated
// - make the above point more solid
// - update parse decision signal to not let short positions be open for spot trades
// - make execution arena
// - unify types like Side, OrderType etc into one
// - standarise order types ie. limit, market etc
// - rm todos
// - mv binance auth to http client
// - impl other user related data methods for execution client
// - change level size to quantity (name change)
// - change r#type to enum instead of string
// - unify auto reconnect script?
