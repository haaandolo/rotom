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
    trader::{
        arb_trader::{self, SpotArbTrader},
        single_trader::SingleMarketTrader,
    },
};
use rotom_oms::{
    event::{Event, EventTx},
    exchange::{
        binance::{
            account_data_stream::BinanaceAccountDataStream, binance_client::BinanceExecution,
        },
        combine_account_data_streams,
        poloniex::{
            account_data_stream::PoloniexAccountDataStream, poloniex_client::PoloniexExecution,
        },
        ExecutionClient,
    },
    execution::{
        arena::spot_arb::{spot_arb_arena::SpotArbArena, spot_arb_executor::SpotArbExecutor},
        simulated::{Config, SimulatedExecution},
        Fees,
    },
    model::{
        order::{AssetFormatted, CancelOrder, OpenOrder, OrderEvent},
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
    let mut order = OrderEvent {
        time: Utc::now(),
        exchange: ExchangeId::BinanceSpot,
        instrument: Instrument::new("op", "usdt"),
        client_order_id: Some(ClientOrderId("9MHbFzZWbL4otb7tzQ5vkZ".to_string())),
        market_meta: MarketMeta {
            close: 1.0,
            time: Utc::now(),
        },
        decision: Decision::Long,
        quantity: 5.0,
        order_kind: rotom_oms::model::OrderKind::Limit,
        order_status: None,
        state: rotom_oms::model::order::OrderState::InTransit,
        filled_gross: 0.0,
    };

    let open_order = OpenOrder::from(&order);
    let cancel_order = CancelOrder::from(&order);

    // // Test Binance Execution
    // let binance_exe = BinanceExecution::new().unwrap();
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
    // let res = binance_exe.open_order(open_order).await;
    // let res = binance_exe
    //     .cancel_order(cancel_order)
    //     .await;
    // let res = binance_exe.cancel_order_all(cancel_order).await;
    // println!("{:#?}", res);
    // binance_exe.receive_responses().await;

    ////////////////////////////////////////////////////
    // Test Poloniex Execution
    // let polo_exe = PoloniexExecution::new().unwrap();
    // let res = polo_exe.open_order(open_order).await;

    // let res = polo_exe.open_order(order.clone()).await;

    // let res = polo_exe
    //     .cancel_order(
    //         "6fa85248-3701-46f4-b1a5-63ae379d2dfe".to_string(),
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
    // let (trader_command_tx, trader_command_rx) = mpsc::channel(10);

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

    //////////////
    // Arb trader
    //////////////
    let op = vec![
        Market::new(ExchangeId::BinanceSpot, Instrument::new("op", "usdt")),
        Market::new(ExchangeId::PoloniexSpot, Instrument::new("op", "usdt")),
    ];

    let arb = vec![
        Market::new(ExchangeId::BinanceSpot, Instrument::new("arb", "usdt")),
        Market::new(ExchangeId::PoloniexSpot, Instrument::new("arb", "usdt")),
    ];

    let market2 = vec![op, arb];

    let (excution_arena_order_event_tx, execution_arena_order_event_rx) = mpsc::unbounded_channel();
    let mut arb_traders = Vec::new();
    let mut trader_command_txs = HashMap::new();
    let mut order_update_txs = HashMap::new();

    for markets in market2.into_iter() {
        // Trade command, to receive commands from the engine
        let (trader_command_tx, trader_command_rx) = mpsc::channel(10);
        trader_command_txs.insert(markets[0].clone(), trader_command_tx); // todo: arb trader has 2 markets

        // Make channels to be able to receive order update from execution arena
        // Since arb trader has 2 assets its tradiding we need to clone the send
        // tx for each asset per exchange. We are using the exchange specific
        // formatting of the asset for the hashmap keys. For example, op_usdt
        // will have a OPUSDT (binance) and OP_USDT (poloniex). Even if 2 exchange
        // have the same asset format it would not matter as the key will just be replaced
        let (order_update_tx, order_update_rx) = mpsc::channel(10);
        for market in markets.clone().into_iter() {
            let asset_formatted = AssetFormatted::from((&market.exchange, &market.instrument));
            order_update_txs.insert(asset_formatted.0, order_update_tx.clone());
        }

        let arb_trader = SpotArbTrader::builder()
            .engine_id(engine_id)
            .market(markets)
            .command_rx(trader_command_rx)
            .event_tx(event_tx.clone())
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
            .send_order_tx(excution_arena_order_event_tx.clone())
            .order_update_rx(order_update_rx)
            .build()
            .unwrap();

        arb_traders.push(arb_trader)
    }

    // Spawn acccount data ws
    tokio::spawn(combine_account_data_streams::<
        BinanaceAccountDataStream,
        PoloniexAccountDataStream,
    >(order_update_txs));

    let spot_arb =
        SpotArbArena::<BinanceExecution, PoloniexExecution>::new(execution_arena_order_event_rx)
            .await;
    tokio::spawn(async move { spot_arb.testing().await });

    // Build engine TODO: (check the commands are doing what it is supposed to)
    // let trader_command_txs = markets
    //     .into_iter()
    //     .map(|market| (market, trader_command_tx.clone()))
    //     .collect::<HashMap<_, _>>();

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

    /////////////////////////////////////////////////
    // Arena
    // let mut arb_exe = SpotArbArena::<BinanceExecution, PoloniexExecution>::new(
    //     execution_arena_order_event_rx,
    //     order_update_txs,
    // )
    // .await;

    // tokio::spawn(async move {
    //     arb_exe.testing().await;
    // });

    // let mut arb_exe = SpotArbExecutor::<BinanceExecution, PoloniexExecution>::init()
    //     .await
    //     .unwrap();

    // while let Some(msg) = arb_exe.streams.recv().await {
    //     println!("{:#?}", msg);
    // }

    // let arb_exe = ArbExecutor::<BinanceExecution, PoloniexExecution>::default();

    // let mut polo_exe = PoloniexExecution::init().await.unwrap();
    // while let Some(message) = polo_exe.rx.recv().await {
    //     println!("--->>> {:#?}", message);
    // }

    // let mut bin_exe = BinanceExecution::init().await.unwrap();
    // while let Some(message) = bin_exe.rx.recv().await {
    //     println!("--->>> {:#?}", message);
    // }

    ///////////////////////////////////////////////////
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
            _ => (),
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
// - maybe impl a trait called "spot arb exe" to limit buy/sell, transfer funds or taker buy/sell for the OrderEvent? and maybe even keeping a order at bba
// - code to keep a limit order at bba
// - start execution function to limit buy -> transfer -> taker sell, i think this should be a function
// - figure out the balance +ve and -ve of quote and base asset for portfolio when the fill is updated
// - make the above point more solid
// - does the balance account data stream for poloniex need to be a Vec<T> or can it be T?
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
