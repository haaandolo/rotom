use chrono::Utc;
use parking_lot::Mutex;
use rotom_data::{
    shared::subscription_models::{ExchangeId, Instrument},
    Market, MarketMeta,
};
use rotom_main::{engine::Engine, trader::spot_arb_trader::builder::SpotArbTradersBuilder};
use rotom_oms::{
    event::EventTx,
    exchange::{
        binance::binance_client::BinanceExecution, poloniex::poloniex_client::PoloniexExecution,
    },
    model::{
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
use rotom_strategy::Decision;
use std::{sync::Arc, time::Duration};
use tokio::sync::mpsc;
use uuid::Uuid;

const ENGINE_RUN_TIMEOUT: Duration = Duration::from_secs(5000);

/*
println!("blocking");
std::thread::sleep(std::time::Duration::from_secs(10));
println!("unblocked");
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

    ////////////////////////////////////////////////////
    // Order
    let mut order = OrderEvent {
        order_request_time: Utc::now(),
        exchange: ExchangeId::BinanceSpot,
        instrument: Instrument::new("op", "usdt"),
        client_order_id: ClientOrderId::random(),
        market_meta: MarketMeta {
            close: 1.81,
            time: Utc::now(),
        },
        decision: Decision::Short,
        original_quantity: 2.8,
        cumulative_quantity: 4.0,
        order_kind: rotom_oms::model::OrderKind::Market,
        exchange_order_status: None,
        internal_order_state: rotom_oms::model::order::OrderState::InTransit,
        filled_gross: 0.0,
        enter_avg_price: 0.0,
        fees: 0.0,
        last_execution_time: None,
    };

    // Requests
    // let open_order = OpenOrder::from(&order);

    let open_order = OpenOrder {
        client_order_id: order.client_order_id,
        price: order.market_meta.close,
        quantity: order.original_quantity,
        notional_amount: order.market_meta.close * order.original_quantity,
        decision: order.decision,
        order_kind: order.order_kind,
        instrument: order.instrument.clone(),
    };

    // let cancel_order = CancelOrder::from(&order);
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
    // Channels
    // Create channel to distribute Commands to the Engine & it's Traders (eg/ Command::Terminate)
    let (_command_tx, command_rx) = mpsc::channel(20);

    // Create channel for each Trader so the Engine can distribute Commands to it
    // let (trader_command_tx, trader_command_rx) = mpsc::channel(10);

    // Create Event channel to listen to all Engine Events in real-time
    let (event_tx, _event_rx) = mpsc::unbounded_channel();
    let _event_tx = EventTx::new(event_tx);

    // Portfolio
    let portfolio = Arc::new(Mutex::new(
        MetaPortfolio::builder()
            .engine_id(engine_id)
            .markets(vec![Market::new(
                ExchangeId::BinanceSpot,
                Instrument::new("op", "usdt"),
            )])
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

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////

    ////////////////////////////////////////////////////
    // Portfolio
    ////////////////////////////////////////////////////
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
    // Execution manager
    //////////////
    // let bin_exe_manager = ExecutionManager {
    //     exeution_client: BinanceExecution::new(),
    // };

    // let polo_exe_manager = ExecutionManager {
    //     exeution_client: PoloniexExecution::new(),
    // };

    //////////////
    // Arb traders builder
    //////////////

    let mut arb_trader_meta = SpotArbTradersBuilder::default()
        .add_traders::<BinanceExecution, PoloniexExecution>(vec![
            Instrument::new("op", "udst"),
            Instrument::new("arb", "udst"),
            Instrument::new("ldo", "udst"),
        ])
        .await;

    let arb_traders = std::mem::take(&mut arb_trader_meta.traders);
    let trader_command_txs = std::mem::take(&mut arb_trader_meta.engine_command_tx);

    /////////////////////////////////////////////////////////////
    // Arena
    /////////////////////////////////////////////////////////////
    // let exchanges = vec![ExchangeId::BinanceSpot, ExchangeId::PoloniexSpot];
    // combine_account_data_stream(exchanges, order_update_txs, Arc::clone(&arb_portfolio)).await;

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
    let _ = tokio::time::timeout(ENGINE_RUN_TIMEOUT, engine.run()).await;
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
// - should there be a middle layer between traders and execution manager?
// - idea for execution client. Wrap execution client in a Arc to generate a future - this should be pretty quick, so won't hold up other threads. Then await it inside the trader to yield it.A
// - also, if we couple a tx and rx for each exchange executionn manager, we can just clone the tx for respective senders
// - make acc data status generic and impl a filled trait
// - how should i update order state after cancel and replace order?
// - figure out limit order exe for polo cos rn its market
// - cancel and replace exe
// - delete strategy class?
// - make separate functions for all execution enum steps in arb trader
// - finish position2, what fields are required for this
// - funcitons to convert orderEvent to OpenOrder, CancelOrder, TransferOrder etc
// - rm lego
// - do we still need balance update in fill updater? for spot portfolio
// - what to do with new order and an existing order exists?
// - make a spot arb specific enum trading loop
// - sometimes account data ws connection connects later than sending an order, so account for this
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
