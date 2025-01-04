use chrono::Utc;
use futures::StreamExt;
use parking_lot::Mutex;
use rotom_data::{
    exchange::{
        binance::public_http::binance_public_http_client::BinancePublicData,
        poloniex::public_http::poloniex_public_http_client::PoloniexPublicData,
    },
    model::{
        market_event::{DataKind, MarketEvent},
        ticker_info::TickerInfo,
    },
    protocols::ws::ws_parser::{StreamParser, WebSocketParser},
    shared::subscription_models::{ExchangeId, Instrument, StreamKind},
    streams::builder::dynamic,
    AssetFormatted, ExchangeAssetId, Market, MarketFeed, MarketMeta,
};
use rotom_main::{
    engine::{self, Engine},
    trader::{
        arb_trader::{self, SpotArbTrader, SpotArbTraderMetaData},
        single_trader::SingleMarketTrader,
    },
};
use rotom_oms::{
    event::{Event, EventTx},
    exchange::{
        binance::{binance_client::BinanceExecution, requests::wallet_transfer},
        combine_account_data_stream,
        poloniex::poloniex_client::PoloniexExecution,
        ExecutionClient,
    },
    execution::{
        simulated::{Config, SimulatedExecution},
        Fees,
    },
    model::{
        account_data::OrderStatus,
        order::{CancelOrder, OpenOrder, OrderEvent, WalletTransfer},
        ClientOrderId,
    },
    portfolio::{
        allocator::{default_allocator::DefaultAllocator, spot_arb_allocator::SpotArbAllocator},
        persistence::{in_memory::InMemoryRepository, spot_in_memory::SpotInMemoryRepository},
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
        order_request_time: Utc::now(),
        exchange: ExchangeId::BinanceSpot,
        instrument: Instrument::new("op", "usdt"),
        client_order_id: Some(ClientOrderId("391776281638920193".to_string())),
        market_meta: MarketMeta {
            close: 1.81,
            time: Utc::now(),
        },
        decision: Decision::Short,
        original_quantity: 5.01,
        cumulative_quantity: 5.01,
        order_kind: rotom_oms::model::OrderKind::Limit,
        exchange_order_status: None,
        internal_order_state: rotom_oms::model::order::OrderState::InTransit,
        filled_gross: 0.0,
        enter_avg_price: 0.0,
        fees: 0.0,
        last_execution_time: None,
    };

    // Requests
    // let open_order = OpenOrder::from(&order);
    let cancel_order = CancelOrder::from(&order);
    let polo_wallet_transfer = WalletTransfer {
        coin: order.instrument.base.clone(),
        wallet_address: "0x1b7c39f6669cee023caff84e06001b03a76f829f".to_string(),
        network: None,
        amount: 4.25,
    };
    let polo_usdt_tron_network = "TBw5BWoS97tWrVr7PSuBtUQeBXU6eJZpyg".to_string();
    let bin_wallet_transfer = WalletTransfer {
        coin: order.instrument.base.clone(),
        // "usdt".to_string(),
        wallet_address: polo_usdt_tron_network,
        // "0xc0b2167fc0ff47fe0783ff6e38c0eecc0f784c2f".to_string(),
        network: None,
        // Some("TRX".to_string()),
        amount: 15.0,
    };

    // // Test Binance Execution
    // let binance_exe = BinanceExecution::new();
    // let res = binance_exe.get_balance_all().await;
    // let res: Vec<AssetBalance> = res.unwrap().into();
    // let res = binance_exe.wallet_transfer(bin_wallet_transfer).await;
    // let res = binance_exe.open_order(open_order).await;
    // let res = binance_exe
    //     .cancel_order(cancel_order)
    //     .await;
    // let res = binance_exe.cancel_order_all(cancel_order).await;
    // println!("{:#?}", res);
    // binance_exe.receive_responses().await;

    ////////////////////////////////////////////////////
    // Test Poloniex Execution
    // let polo_exe = PoloniexExecution::new();
    // println!("---> open order res: {:#?}", open_order);
    // let res = polo_exe.open_order(open_order).await;
    // let res = polo_exe.open_order(order.clone()).await;
    // let res = polo_exe.cancel_order(cancel_order).await;
    // let res= polo_exe.cancel_order_all("OP_USDT".to_string()).await;
    // polo_exe.receive_responses().await;
    // let res = polo_exe.wallet_transfer(polo_wallet_transfer).await;
    // println!("---> {:#?}", res);

    ////////////////////////////////////////////////
    /*----- */
    // Trader builder
    /*----- */
    /////////////////////////////////////////////////
    // Engine id
    let engine_id = Uuid::new_v4();

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

    ////////////////////////////////////////////////////
    // Portfolio
    let arb_portfolio = Arc::new(Mutex::new(
        SpotPortfolio::new(
            engine_id,
            vec![ExchangeId::BinanceSpot, ExchangeId::PoloniexSpot],
            SpotInMemoryRepository::default(),
            SpotArbAllocator,
        )
        .init()
        .await
        .unwrap(),
    ));

    println!("arb portfolio: {:#?}", arb_portfolio);
    //////////////
    // Arb trader
    //////////////
    let op = vec![
        Market::new(ExchangeId::BinanceSpot, Instrument::new("op", "usdt")),
        Market::new(ExchangeId::PoloniexSpot, Instrument::new("op", "usdt")),
    ];

    // let arb = vec![
    //     Market::new(ExchangeId::BinanceSpot, Instrument::new("arb", "usdt")),
    //     Market::new(ExchangeId::PoloniexSpot, Instrument::new("arb", "usdt")),
    // ];

    let market2 = vec![op];
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
        let mut spot_arb_meta_data = SpotArbTraderMetaData::default();
        for market in markets.clone().into_iter() {
            // Key to update order
            let order_update_key = ExchangeAssetId::from((&market.exchange, &market.instrument));
            order_update_txs.insert(order_update_key, order_update_tx.clone());

            // Key to update balances
            let balance_update_key = ExchangeAssetId(
                format!("{}_{}", &market.exchange.as_str(), &market.instrument.base).to_uppercase(),
            );
            order_update_txs.insert(balance_update_key, order_update_tx.clone());

            // Get ticker precision
            let liquid_ticker_info = BinancePublicData::get_ticker_info(&market.instrument)
                .await
                .unwrap();
            spot_arb_meta_data.liquid_ticker_info = TickerInfo::from(liquid_ticker_info);

            let mut illiquid_ticker_info = PoloniexPublicData::get_ticker_info(&market.instrument)
                .await
                .unwrap();
            spot_arb_meta_data.illiquid_ticker_info =
                TickerInfo::from(illiquid_ticker_info.remove(0));

            // todo: get wallet addresses http requests?
            spot_arb_meta_data.liquid_deposit_address =
                "0x1b7c39f6669cee023caff84e06001b03a76f829f".to_string();
            spot_arb_meta_data.illiquid_deposit_address =
                "0xc0b2167fc0ff47fe0783ff6e38c0eecc0f784c2f".to_string();
        }

        let arb_trader = SpotArbTrader::builder()
            .engine_id(engine_id)
            .market(markets)
            .command_rx(trader_command_rx)
            .event_tx(event_tx.clone())
            .portfolio(Arc::clone(&arb_portfolio))
            .data(MarketFeed::new(stream_trades().await))
            .strategy(SpreadStategy::new())
            .liquid_exchange(BinanceExecution::new())
            .illiquid_exchange(PoloniexExecution::new())
            .order_update_rx(order_update_rx)
            .meta_data(spot_arb_meta_data)
            .build()
            .unwrap();

        arb_traders.push(arb_trader)
    }

    /////////////////////////////////////////////////////////////
    // Arena
    /////////////////////////////////////////////////////////////
    let exchanges = vec![ExchangeId::BinanceSpot, ExchangeId::PoloniexSpot];
    combine_account_data_stream(exchanges, order_update_txs, Arc::clone(&arb_portfolio)).await;

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
// - figure out limit order exe for polo cos rn its market
// - cancel and replace exe
// - make separate functions for all execution enum steps in arb trader
// - finish position2, what fields are required for this
// - funcitons to convert orderEvent to OpenOrder, CancelOrder, TransferOrder etc
// - do we still need balance update in fill updater? for spot portfolio
// - what to do with new order and an existing order exists?
// - rate limit ring buffer
// - code to keep a limit order at bba
// - make binance account stream get keys automatically every hour
// - start execution function to limit buy -> transfer -> taker sell, i think this should be a function
// - figure out the balance +ve and -ve of quote and base asset for portfolio when the fill is updated
// - make the above point more solid
// - update parse decision signal to not let short positions be open for spot trades
// - rm todos
// - mv binance auth to http client
// - impl other user related data methods for execution client
// - change level size to quantity (name change)
// - change r#type to enum instead of string
// - unify auto reconnect script?

/*
# Execution key points
- 2 versions of execution, buy illiquid & sell illiquid
- always taker buy and sell at the liquid exchange
- always maker buy and sell at bba or deeper at the illiquid exchange

# Buy illiquid
- limit order at bba illiquid exchange
- transfer funds to liquid exchange
- taker out the position

# Sell illiquid
- taker order buy on the liquid exchange
- transfer funds to illiquid exchange
- sell out using limit order at bba in the illiquid exchange
*/

/*
do we still need binance delta update if this is happening?
SpotBalanceId(
    "binancespot_op",
)
>>>>>>> before delta update >>>>>>>>>>
InMemoryRepository2 {
    open_positions: {},
    closed_positions: {},
    current_balance: {
        SpotBalanceId(
            "poloniexspot_op",
        ): Balance {
            total: 0.4132594,
            available: 0.0,
        },
        SpotBalanceId(
            "binancespot_op",
        ): Balance {
            total: 1.9,
            available: 0.0,
        },
        SpotBalanceId(
            "binancespot_usdce",
        ): Balance {
            total: 1.0,
            available: 0.0,
        },
        SpotBalanceId(
            "binancespot_usdt",
        ): Balance {
            total: 5.05022655,
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
            total: 10.57642853676,
            available: 0.0,
        },
    },
}
balance: 1.9
balance delta: 0.2
>>>>>>> after delta update >>>>>>>>>>
InMemoryRepository2 {
    open_positions: {},
    closed_positions: {},
    current_balance: {
        SpotBalanceId(
            "poloniexspot_op",
        ): Balance {
            total: 0.4132594,
            available: 0.0,
        },
        SpotBalanceId(
            "binancespot_op",
        ): Balance {
            total: 2.1,
            available: 0.0,
        },
        SpotBalanceId(
            "binancespot_usdce",
        ): Balance {
            total: 1.0,
            available: 0.0,
        },
        SpotBalanceId(
            "binancespot_usdt",
        ): Balance {
            total: 5.05022655,
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
            total: 10.57642853676,
            available: 0.0,
        },
    },
}
AccountData: AccountDataBalanceDelta {
    asset: "OP",
    exchange: BinanceSpot,
    total: 0.2,
    available: 0.0,
}
AccountData: AccountDataBalance {
    asset: "OP",
    exchange: BinanceSpot,
    balance: Balance {
        total: 2.1,
        available: 0.0,
    },
}
*/
