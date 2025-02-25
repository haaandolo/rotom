use std::{collections::HashMap, fs::File, io::Write, sync::Arc};

use futures::StreamExt;
use parking_lot::RwLock;
use rotom_data::{
    exchange::{
        ascendex::AscendExSpotPublicData,
        binance::BinanceSpotPublicData,
        coinex::{market::CoinExMarket, CoinExSpotPublicData},
        exmo::ExmoSpotPublicData,
        htx::HtxSpotPublicData,
        kucoin::KuCoinSpotPublicData,
        okx::OkxSpotPublicData,
        phemex::PhemexSpotPublicData,
        woox::WooxSpotPublicData,
        PublicHttpConnector,
    },
    model::{
        event_book_snapshot::OrderBookSnapshot,
        market_event::{DataKind, MarketEvent},
        network_info::NetworkSpecs,
    },
    playground::test_http,
    shared::subscription_models::{ExchangeId, Instrument, StreamKind},
    streams::dynamic_stream::DynamicStreams,
    MarketFeed,
};
use rotom_main::trader::spot_arb_trader::builder::stream_trades;
use rotom_oms::exchange::{
    binance::binance_client::BinanceExecution, poloniex::poloniex_client::PoloniexExecution,
};
use tokio::sync::mpsc;
use tracing_subscriber::fmt::init;

#[tokio::main]
pub async fn main() {
    ///////////
    // Main
    ///////////
    init_logging();

    // let res = test_http().await;
    // let res = AscendExSpotPublicData::get_usdt_pair().await;
    // let res = HtxSpotPublicData::get_usdt_pair().await;
    // let res = BinanceSpotPublicData::get_usdt_pair().await;
    // let res = OkxSpotPublicData::get_usdt_pair().await;
    // let res = CoinExSpotPublicData::get_usdt_pair().await;
    // let res = ExmoSpotPublicData::get_usdt_pair().await;
    let res = KuCoinSpotPublicData::get_usdt_pair().await;
    // let res = PhemexSpotPublicData::get_usdt_pair().await;
    // let res = WooxSpotPublicData::get_usdt_pair().await;

    // println!("{:#?}", res);

    // let test = OkxSpotPublicData::get_network_info(instruments)
    //     .await
    //     .unwrap();

    // let test2: NetworkSpecs = test.into();
    // println!("network {:#?}", test2);

    // let mut test2 = Vec::new();
    // for bin in test.into_iter() {
    //     for network in bin.network_list.into_iter() {
    //         test2.push((network.network, network.estimated_arrival_time));
    //     }
    // }

    // test2.sort();
    // test2.dedup();
    // println!("{:#?}", test2);

    // let mut file = File::create("./okx_network_info.json").unwrap();
    // let json_string = serde_json::to_string_pretty(&test).unwrap();
    // file.write_all(json_string.as_bytes()).unwrap();

    /////////
    // Dynamic stream
    /////////
    // let streams = DynamicStreams::init([vec![
    //     // (ExchangeId::ExmoSpot, "trx", "usdt", StreamKind::Snapshot),
    //     // (ExchangeId::AscendExSpot, "btc", "usdt", StreamKind::Trades),
    //     // (ExchangeId::AscendExSpot, "eth", "usdt", StreamKind::Trades),
    //     // (ExchangeId::ExmoSpot, "eth", "usdt", StreamKind::Snapshot),
    //     // (ExchangeId::PhemexSpot, "btc", "usdt", StreamKind::Snapshot),
    //     (ExchangeId::PhemexSpot, "ada", "usdt", StreamKind::Snapshot),
    //     // (ExchangeId::ExmoSpot, "xrp", "usdt", StreamKind::Trades),
    //     // (ExchangeId::KuCoinSpot, "btc", "usdt", StreamKind::Trade),
    //     // (ExchangeId::HtxSpot, "sol", "usdt", StreamKind::Trades),
    // ]])
    // .await
    // .unwrap();

    // let mut merged = streams.select_all::<MarketEvent<DataKind>>();
    // while let Some(event) = merged.next().await {
    //     // println!("{:?}", event);
    //     // println!("###########");
    // }

    ///////////
    // Testing
    ///////////
    // let mut rx =
    //     stream_trades::<BinanceExecution, PoloniexExecution>(&Instrument::new("btc", "usdt")).await;
    // while let Some(msg) = rx.recv().await {
    //     println!("{:?}", msg);
    // }
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

// Todo:
// Okx checkout bonk_usdt and agld_usdt to see if stream comes in
// last update info for each stream
// Ascendex too many requests, need something to stop sending too many requests
// write test
// Change coin in NetworkSpecData to be a type not string
// Http error handling for get network publichttpclient
// these exchanges have transfer times: binance, kucoin, okx, htx,
// figure out scanner archtecture e.g. streaming in chain data every 10 mins
// bitstamp chain info
// change instrument map key from market to stream key

//////////////////////////////////////////////////////////////////////////////////////////////
// Execution - WIP
//////////////////////////////////////////////////////////////////////////////////////////////
// use chrono::Utc;
// use parking_lot::Mutex;
// use rotom_data::{
//     error::SocketError,
//     exchange::{
//         self, binance::BinanceSpotPublicData, poloniex::PoloniexSpotPublicData, PublicHttpConnector,
//     },
//     shared::subscription_models::{ExchangeId, Instrument},
//     Market, MarketMeta,
// };
// use rotom_main::{engine::Engine, trader::spot_arb_trader::builder::SpotArbTradersBuilder};
// use rotom_oms::{
//     event::EventTx,
//     exchange::{
//         binance::binance_client::BinanceExecution,
//         poloniex::{
//             poloniex_client::PoloniexExecution,
//             requests::wallet_transfer::PoloniexWalletTransferResponse,
//         },
//         ExecutionClient,
//     },
//     execution_manager::builder::{ExecutionBuilder, TraderId},
//     model::{ClientOrderId, OrderKind},
//     order_manager::{balance_builder::BalanceBuilder, manager::OrderManagementSystemBuilder},
//     portfolio::{
//         allocator::{default_allocator::DefaultAllocator, spot_arb_allocator::SpotArbAllocator},
//         persistence::{in_memory::InMemoryRepository, spot_in_memory::SpotInMemoryRepository},
//         portfolio_type::{default_portfolio::MetaPortfolio, spot_portfolio::SpotPortfolio},
//         risk_manager::default_risk_manager::DefaultRisk,
//     },
//     statistic::summary::{
//         trading::{self, Config as StatisticConfig, TradingSummary},
//         Initialiser,
//     },
// };
// use rotom_strategy::Decision;
// use std::{sync::Arc, time::Duration};
// use tokio::sync::mpsc;
// use uuid::Uuid;

// /*
// println!("blocking");
// std::thread::sleep(std::time::Duration::from_secs(10));
// println!("unblocked");
// */
// const ENGINE_RUN_TIMEOUT: Duration = Duration::from_secs(5000);
// /*----- */
// // Main
// /*----- */
// #[tokio::main]
// pub async fn main() {
//     // Initialise logging
//     init_logging();

//     // Engine id
//     let engine_id = Uuid::new_v4();

//     ////////////////////////////////////////////////
//     // Old Portfolio - to be replaced
//     ///////////////////////////////////////////////////
//     let portfolio = Arc::new(Mutex::new(
//         MetaPortfolio::builder()
//             .engine_id(engine_id)
//             .markets(vec![Market::new(
//                 ExchangeId::BinanceSpot,
//                 Instrument::new("op", "usdt"),
//             )])
//             .starting_cash(10000.0)
//             .repository(InMemoryRepository::<TradingSummary>::new())
//             .allocation_manager(DefaultAllocator {
//                 default_order_value: 100.0,
//             })
//             .risk_manager(DefaultRisk {})
//             .statistic_config(StatisticConfig {
//                 starting_equity: 10_000.0,
//                 trading_days_per_year: 365,
//                 risk_free_return: 0.0,
//             })
//             .build_init()
//             .unwrap(),
//     ));

//     ////////////////////////////////////////////////////////////////////////////////////////////////////
//     ////////////////////////////////////////////////////////////////////////////////////////////////////

//     ////////////////////////////////////////////////////
//     // Init channels and other
//     ////////////////////////////////////////////////////
//     // Tx goes to Traders and rx goes to oms
//     let (execution_request_tx, execution_request_rx) = mpsc::unbounded_channel();

//     // Tx goes to ExecutionManagers, rx goes to oms
//     let (execution_response_tx, execution_response_rx) = mpsc::unbounded_channel();

//     // Declare trading universe
//     let trading_universe = vec![
//         Instrument::new("op", "usdt"),
//         // Instrument::new("arb", "usdt"),
//         // Instrument::new("ldo", "usdt"),
//     ];

//     ////////////////////////////////////////////////////
//     // Balance builder
//     ////////////////////////////////////////////////////
//     let balances = BalanceBuilder::default()
//         .add_exchange::<BinanceExecution>()
//         .add_exchange::<PoloniexExecution>()
//         .build()
//         .await
//         .unwrap();

//     println!("{:#?}", balances);

//     ////////////////////////////////////////////////////
//     // Execution manager
//     ////////////////////////////////////////////////////
//     let execution_manager_txs = ExecutionBuilder::default()
//         .add_exchange::<BinanceExecution>(execution_response_tx.clone(), trading_universe.clone())
//         .await
//         .add_exchange::<PoloniexExecution>(execution_response_tx.clone(), trading_universe.clone())
//         .await
//         .build();

//     ////////////////////////////////////////////////////
//     // Arb traders builder
//     ////////////////////////////////////////////////////
//     let (arb_traders, trader_command_txs, execution_response_txs) =
//         SpotArbTradersBuilder::new(execution_request_tx.clone())
//             .add_traders::<BinanceExecution, PoloniexExecution>(vec![
//                 Instrument::new("op", "usdt"),
//                 // Instrument::new("arb", "usdt"),
//                 // Instrument::new("ldo", "usdt"),
//             ])
//             .await
//             .build();

//     ////////////////////////////////////////////////////
//     // OMS builder
//     ////////////////////////////////////////////////////
//     let oms = OrderManagementSystemBuilder::default()
//         .balances(balances)
//         .execution_request_rx(execution_request_rx)
//         .execution_manager_txs(execution_manager_txs)
//         .execution_response_rx(execution_response_rx)
//         .execution_response_txs(execution_response_txs)
//         .build()
//         .unwrap();

//     tokio::spawn(async move { oms.run().await });

//     ////////////////////////////////////////////////////
//     // Engine
//     ////////////////////////////////////////////////////
//     // Create channel to distribute Commands to the Engine & it's Traders (eg/ Command::Terminate)
//     let (_command_tx, command_rx) = mpsc::channel(20);

//     let engine = Engine::builder()
//         .engine_id(engine_id)
//         .command_rx(command_rx)
//         .portfolio(portfolio)
//         .traders(arb_traders)
//         .trader_command_txs(trader_command_txs)
//         .statistics_summary(TradingSummary::init(StatisticConfig {
//             starting_equity: 1000.0,
//             trading_days_per_year: 365,
//             risk_free_return: 0.0,
//         }))
//         .build()
//         .expect("failed to build engine");

//     let _ = tokio::time::timeout(ENGINE_RUN_TIMEOUT, engine.run()).await;

//     //////////////////////////////////////////////////////////////////////////////////////////////////////
//     //////////////////////////////////////////////////////////////////////////////////////////////////////

//     // ////////////////////////////////////////////////////
//     // // Testing execution
//     // ////////////////////////////////////////////////////

//     // let execution_tx_map = ExecutionBuilder::default()
//     //     .add_exchange::<BinanceExecution>()
//     //     .add_exchange::<PoloniexExecution>()
//     //     .build();

//     // println!("blocking");
//     // std::thread::sleep(std::time::Duration::from_secs(5));
//     // println!("unblocked");

//     // let trader_id = TraderId(uuid::Uuid::new_v4());
//     // let client_order_id = ClientOrderId::random();
//     // let instrument = Instrument::new("icp", "usdt");
//     // let bin_symbol = String::from("ICPUSDT");
//     // let polo_symbol = String::from("ICP_USDT");
//     // let price = 10.82;
//     // let quantity = 0.49;
//     // let notional = price * quantity;

//     // // Requests
//     // let open_order = OpenOrder {
//     //     trader_id,
//     //     client_order_id: client_order_id.clone(),
//     //     price,
//     //     quantity,
//     //     notional_amount: round_float_to_precision(notional, 0.01),
//     //     decision: Decision::Long,
//     //     order_kind: OrderKind::Limit,
//     //     instrument: instrument.clone(),
//     // };

//     // let cancel_order = CancelOrder {
//     //     trader_id,
//     //     client_order_id: client_order_id.clone().0,
//     //     symbol: polo_symbol,
//     // };

//     // let polo_wallet_transfer = WalletTransfer {
//     //     trader_id: TraderId(uuid::Uuid::new_v4()),
//     //     coin: "op".to_string(),
//     //     wallet_address: "0x1b7c39f6669cee023caff84e06001b03a76f829f".to_string(),
//     //     network: None,
//     //     amount: 3.91,
//     // };

//     // let bin_wallet_transfer = WalletTransfer {
//     //     trader_id: TraderId(uuid::Uuid::new_v4()),
//     //     coin: "op".to_string(),
//     //     wallet_address: "0xc0b2167fc0ff47fe0783ff6e38c0eecc0f784c2f".to_string(),
//     //     network: None,
//     //     amount: 4.27,
//     // };

//     // println!("request: {:#?}", open_order);

//     // // Test Binance Execution
//     // let binance_exe = BinanceExecution::new();
//     // let res = binance_exe.get_balance_all().await;
//     // let res: Vec<AssetBalance> = res.unwrap().into();
//     // let res = binance_exe.wallet_transfer(bin_wallet_transfer).await;
//     // let res = binance_exe.open_order(open_order).await;
//     // let res = binance_exe
//     //     .cancel_order(cancel_order)
//     //     .await;
//     // let res = binance_exe.cancel_order_all(cancel_order).await;
//     // println!("{:#?}", res);

//     ////////////////////////////////////////////////////
//     // Test Poloniex Execution
//     // let polo_exe = PoloniexExecution::new();
//     // println!("---> open order res: {:#?}", open_order);
//     // let res = polo_exe.open_order(open_order).await;
//     // let res = polo_exe.open_order(order.clone()).await;
//     // let res = polo_exe.cancel_order(cancel_order).await;
//     // let res= polo_exe.cancel_order_all("OP_USDT".to_string()).await;
//     // polo_exe.receive_responses().await;
//     // let res: Result<PoloniexWalletTransferResponse, SocketError> =
//     // let res = polo_exe.wallet_transfer(polo_wallet_transfer).await;
//     // println!("---> {:#?}", res);

//     // loop {}
// }

// /*----- */
// // Logging config
// /*----- */
// fn init_logging() {
//     tracing_subscriber::fmt()
//         .with_env_filter(
//             tracing_subscriber::filter::EnvFilter::builder()
//                 .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
//                 .from_env_lossy(),
//         )
//         // Disable colours on release builds
//         .with_ansi(cfg!(debug_assertions))
//         // Enable Json formatting
//         .json()
//         // Install this Tracing subscriber as global default
//         .init()
// }

// /*----- */
// // Todo
// /*----- */
// // - look at bin transfers again and do the rest of poloniex
// // - work on exeution manager
// // - make acc data status generic and impl a filled trait
// // - only deserialise id for http response
// // - how should i update order state after cancel and replace order?
// // - figure out limit order exe for polo cos rn its market
// // - delete strategy class?
// // - finish position2, what fields are required for this
// // - rm lego
// // - spot arb traders builder stream should be not hard coded and try rm the await in this func
// // - do we still need balance update in fill updater? for spot portfolio
// // - make a spot arb specific enum trading loop
// // - sometimes account data ws connection connects later than sending an order, so account for this
// // - rate limit ring buffer
// // - code to keep a limit order at bba
// // - make binance account stream get keys automatically every hour
// // - rm todos
// // - change level size to quantity (name change)
// // - change r#type to enum instead of string
// // - unify auto reconnect script?

// /*
// # Execution key points
// - 2 versions of execution, buy illiquid & sell illiquid
// - always taker buy and sell at the liquid exchange
// - always maker buy and sell at bba or deeper at the illiquid exchange

// # Buy illiquid
// - limit order at bba illiquid exchange
// - transfer funds to liquid exchange
// - taker out the position

// # Sell illiquid
// - taker order buy on the liquid exchange
// - transfer funds to illiquid exchange
// - sell out using limit order at bba in the illiquid exchange
// */
// /*----- */
// // Binance
// /*----- */
// /*
// ##################
// ### Market Buy ###
// ##################

// SpotBalanceId(
//     "binancespot_steem",
// ): Balance {
//     total: 23.0769,
//     available: 0.0,
// }

// SpotBalanceId(
//     "binancespot_usdt",
// ): Balance {
//     total: 8.46120048,
//     available: 0.0,
// }

// OpenOrder {
//     trader_id: TraderId(
//         7eb5ebe2-7f2f-4f27-8375-17fb9950b0db,
//     ),
//     client_order_id: ClientOrderId(
//         "aJIrGFA8QetM2_AVlQd1GoQ",
//     ),
//     price: 0.252,
//     quantity: 23.1,
//     notional_amount: 5.8212,
//     decision: Long,
//     order_kind: Market,
//     instrument: Instrument {
//         base: "steem",
//         quote: "usdt",
//     },
// }

// # Http response
// BinanceNewOrderResponseFull {
//     symbol: "STEEMUSDT",
//     order_id: 394510025,
//     order_list_id: -1,
//     client_order_id: "aJIrGFA8QetM2_AVlQd1GoQ",
//     transact_time: 1736918698514,
//     price: 0.0,
//     orig_qty: 23.1,
//     executed_qty: 23.1,
//     cummulative_quote_qty: 5.83737,
//     status: Filled,
//     time_in_force: GTC,
//     type: "MARKET",
//     side: "BUY",
//     working_time: 1736918698514,
//     fills: [
//         BinanceFill {
//             price: 0.2527,
//             qty: 23.1,
//             commission: 0.0231,
//             commission_asset: "STEEM",
//             trade_id: 19443260,
//         },
//     ],
// }

// BinanceAccountDataOrder {
//     e: "executionReport",
//     E: 1736918698515,
//     s: "STEEMUSDT",
//     c: "aJIrGFA8QetM2_AVlQd1GoQ",
//     S: Buy,
//     o: Market,
//     f: GTC,
//     q: 23.1,
//     p: 0.0,
//     P: 0.0,
//     F: 0.0,
//     g: -1,
//     C: "",
//     x: New,
//     X: New,
//     r: "NONE",
//     i: 394510025,
//     l: 0.0,
//     z: 0.0,
//     L: 0.0,
//     n: 0.0,
//     N: None,
//     T: 2025-01-15T05:24:58.514Z,
//     t: -1,
//     v: None,
//     I: 807448936,
//     w: true,
//     m: false,
//     M: false,
//     O: 2025-01-15T05:24:58.514Z,
//     Z: 0.0,
//     Y: 0.0,
//     Q: 0.0,
//     W: 1736918698514,
//     V: "EXPIRE_MAKER",
// }

// BinanceAccountDataOrder {
//     e: "executionReport",
//     E: 1736918698515,
//     s: "STEEMUSDT",
//     c: "aJIrGFA8QetM2_AVlQd1GoQ",
//     S: Buy,
//     o: Market,
//     f: GTC,
//     q: 23.1,
//     p: 0.0,
//     P: 0.0,
//     F: 0.0,
//     g: -1,
//     C: "",
//     x: Trade,
//     X: Filled,
//     r: "NONE",
//     i: 394510025,
//     l: 23.1,
//     z: 23.1,
//     L: 0.2527,
//     n: 0.0231,
//     N: Some(
//         "STEEM",
//     ),
//     T: 2025-01-15T05:24:58.514Z,
//     t: 19443260,
//     v: None,
//     I: 807448937,
//     w: false,
//     m: false,
//     M: true,
//     O: 2025-01-15T05:24:58.514Z,
//     Z: 5.83737,
//     Y: 5.83737,
//     Q: 0.0,
//     W: 1736918698514,
//     V: "EXPIRE_MAKER",
// },

// BinanceAccountDataBalance {
//     e: "outboundAccountPosition",
//     E: 1736918698515,
//     u: 1736918698514,
//     B: [
//         BinanceAccountDataBalanceVec {
//             a: "BNB",
//             f: 0.0,
//             l: 0.0,
//         },
//         BinanceAccountDataBalanceVec {
//             a: "USDT",
//             f: 2.62383048,
//             l: 0.0,
//         },
//         BinanceAccountDataBalanceVec {
//             a: "STEEM",
//             f: 46.1538,
//             l: 0.0,
//         },
//     ],
// }

// ###################
// ### Market Sell ###
// ###################

// SpotBalanceId(
//     "binancespot_steem",
// ): Balance {
//     total: 46.1538,
//     available: 0.0,
// }

// SpotBalanceId(
//     "binancespot_usdt",
// ): Balance {
//     total: 2.62383048,
//     available: 0.0,
// }

// OpenOrder {
//     trader_id: TraderId(
//         d246a45f-3b16-4f5e-a820-1a9bd9ad6b20,
//     ),
//     client_order_id: ClientOrderId(
//         "oi4DwPhLboMxD2FzKJ1lUn_",
//     ),
//     price: 0.252,
//     quantity: 46.1,
//     notional_amount: 11.6172,
//     decision: Short,
//     order_kind: Market,
//     instrument: Instrument {
//         base: "steem",
//         quote: "usdt",
//     },
// }

// BinanceNewOrderResponseFull {
//     symbol: "STEEMUSDT",
//     order_id: 394513715,
//     order_list_id: -1,
//     client_order_id: "oi4DwPhLboMxD2FzKJ1lUn_",
//     transact_time: 1736918948380,
//     price: 0.0,
//     orig_qty: 46.1,
//     executed_qty: 46.1,
//     cummulative_quote_qty: 11.68174,
//     status: Filled,
//     time_in_force: GTC,
//     type: "MARKET",
//     side: "SELL",
//     working_time: 1736918948380,
//     fills: [
//         BinanceFill {
//             price: 0.2534,
//             qty: 46.1,
//             commission: 0.01168174,
//             commission_asset: "USDT",
//             trade_id: 19443354,
//         },
//     ],
// }

// BinanceAccountDataOrder {
//     e: "executionReport",
//     E: 1736918948380,
//     s: "STEEMUSDT",
//     c: "oi4DwPhLboMxD2FzKJ1lUn_",
//     S: Sell,
//     o: Market,
//     f: GTC,
//     q: 46.1,
//     p: 0.0,
//     P: 0.0,
//     F: 0.0,
//     g: -1,
//     C: "",
//     x: New,
//     X: New,
//     r: "NONE",
//     i: 394513715,
//     l: 0.0,
//     z: 0.0,
//     L: 0.0,
//     n: 0.0,
//     N: None,
//     T: 2025-01-15T05:29:08.380Z,
//     t: -1,
//     v: None,
//     I: 807456414,
//     w: true,
//     m: false,
//     M: false,
//     O: 2025-01-15T05:29:08.380Z,
//     Z: 0.0,
//     Y: 0.0,
//     Q: 0.0,
//     W: 1736918948380,
//     V: "EXPIRE_MAKER",
// }

// BinanceAccountDataOrder {
//     e: "executionReport",
//     E: 1736918948380,
//     s: "STEEMUSDT",
//     c: "oi4DwPhLboMxD2FzKJ1lUn_",
//     S: Sell,
//     o: Market,
//     f: GTC,
//     q: 46.1,
//     p: 0.0,
//     P: 0.0,
//     F: 0.0,
//     g: -1,
//     C: "",
//     x: Trade,
//     X: Filled,
//     r: "NONE",
//     i: 394513715,
//     l: 46.1,
//     z: 46.1,
//     L: 0.2534,
//     n: 0.01168174,
//     N: Some(
//         "USDT",
//     ),
//     T: 2025-01-15T05:29:08.380Z,
//     t: 19443354,
//     v: None,
//     I: 807456415,
//     w: false,
//     m: false,
//     M: true,
//     O: 2025-01-15T05:29:08.380Z,
//     Z: 11.68174,
//     Y: 11.68174,
//     Q: 0.0,
//     W: 1736918948380,
//     V: "EXPIRE_MAKER",
// }

// BinanceAccountDataBalance {
//     e: "outboundAccountPosition",
//     E: 1736918948380,
//     u: 1736918948380,
//     B: [
//         BinanceAccountDataBalanceVec {
//             a: "BNB",
//             f: 0.0,
//             l: 0.0,
//         },
//         BinanceAccountDataBalanceVec {
//             a: "USDT",
//             f: 14.29388874,
//             l: 0.0,
//         },
//         BinanceAccountDataBalanceVec {
//             a: "STEEM",
//             f: 0.0538,
//             l: 0.0,
//         },
//     ],
// }

// #################
// ### Limit Buy ###
// #################

// SpotBalanceId(
//     "binancespot_steem",
// ): Balance {
//     total: 0.0538,
//     available: 0.0,
// }

// SpotBalanceId(
//     "binancespot_usdt",
// ): Balance {
//     total: 14.29388874,
//     available: 0.0,
// }

// OpenOrder {
//     trader_id: TraderId(
//         dead3e68-34e8-4121-9a92-6be6d09b29db,
//     ),
//     client_order_id: ClientOrderId(
//         "4Mbi-0nVXbC8eTl_z8BYJTQ",
//     ),
//     price: 0.2538,
//     quantity: 36.1,
//     notional_amount: 9.162180000000001,
//     decision: Long,
//     order_kind: Limit,
//     instrument: Instrument {
//         base: "steem",
//         quote: "usdt",
//     },
// }

// # Http res
// BinanceNewOrderResponseFull {
//     symbol: "STEEMUSDT",
//     order_id: 394518763,
//     order_list_id: -1,
//     client_order_id: "4Mbi-0nVXbC8eTl_z8BYJTQ",
//     transact_time: 1736919277053,
//     price: 0.2538,
//     orig_qty: 36.1,
//     executed_qty: 0.0,
//     cummulative_quote_qty: 0.0,
//     status: New,
//     time_in_force: GTC,
//     type: "LIMIT",
//     side: "BUY",
//     working_time: 1736919277053,
//     fills: [],
// }

// BinanceAccountDataOrder {
//     e: "executionReport",
//     E: 1736919277053,
//     s: "STEEMUSDT",
//     c: "4Mbi-0nVXbC8eTl_z8BYJTQ",
//     S: Buy,
//     o: Limit,
//     f: GTC,
//     q: 36.1,
//     p: 0.2538,
//     P: 0.0,
//     F: 0.0,
//     g: -1,
//     C: "",
//     x: New,
//     X: New,
//     r: "NONE",
//     i: 394518763,
//     l: 0.0,
//     z: 0.0,
//     L: 0.0,
//     n: 0.0,
//     N: None,
//     T: 2025-01-15T05:34:37.053Z,
//     t: -1,
//     v: None,
//     I: 807466626,
//     w: true,
//     m: false,
//     M: false,
//     O: 2025-01-15T05:34:37.053Z,
//     Z: 0.0,
//     Y: 0.0,
//     Q: 0.0,
//     W: 1736919277053,
//     V: "EXPIRE_MAKER",
// },

// BinanceAccountDataBalance {
//     e: "outboundAccountPosition",
//     E: 1736919277053,
//     u: 1736919277053,
//     B: [
//         BinanceAccountDataBalanceVec {
//             a: "BNB",
//             f: 0.0,
//             l: 0.0,
//         },
//         BinanceAccountDataBalanceVec {
//             a: "USDT",
//             f: 5.13170874,
//             l: 9.16218,
//         },
//         BinanceAccountDataBalanceVec {
//             a: "STEEM",
//             f: 0.0538,
//             l: 0.0,
//         },
//     ],
// },

// BinanceAccountDataOrder {
//     e: "executionReport",
//     E: 1736919302767,
//     s: "STEEMUSDT",
//     c: "4Mbi-0nVXbC8eTl_z8BYJTQ",
//     S: Buy,
//     o: Limit,
//     f: GTC,
//     q: 36.1,
//     p: 0.2538,
//     P: 0.0,
//     F: 0.0,
//     g: -1,
//     C: "",
//     x: Trade,
//     X: Filled,
//     r: "NONE",
//     i: 394518763,
//     l: 36.1,
//     z: 36.1,
//     L: 0.2538,
//     n: 0.0361,
//     N: Some(
//         "STEEM",
//     ),
//     T: 2025-01-15T05:35:02.767Z,
//     t: 19443502,
//     v: None,
//     I: 807466784,
//     w: false,
//     m: true,
//     M: true,
//     O: 2025-01-15T05:34:37.053Z,
//     Z: 9.16218,
//     Y: 9.16218,
//     Q: 0.0,
//     W: 1736919277053,
//     V: "EXPIRE_MAKER",
// }

// BinanceAccountDataBalance {
//     e: "outboundAccountPosition",
//     E: 1736919302767,
//     u: 1736919302767,
//     B: [
//         BinanceAccountDataBalanceVec {
//             a: "BNB",
//             f: 0.0,
//             l: 0.0,
//         },
//         BinanceAccountDataBalanceVec {
//             a: "USDT",
//             f: 5.13170874,
//             l: 0.0,
//         },
//         BinanceAccountDataBalanceVec {
//             a: "STEEM",
//             f: 36.1177,
//             l: 0.0,
//         },
//     ],
// },

// ##################
// ### Limit Sell ###
// ##################

// SpotBalanceId(
//     "binancespot_usdt",
// ): Balance {
//     total: 5.13170874,
//     available: 0.0,
// }

// SpotBalanceId(
//     "binancespot_steem",
// ): Balance {
//     total: 36.1177,
//     available: 0.0,
// },

// OpenOrder {
//     trader_id: TraderId(
//         25325855-76a7-4118-afcb-cfa124bbf222,
//     ),
//     client_order_id: ClientOrderId(
//         "32Pmn--j1PD3YC-UiK8vhRn",
//     ),
//     price: 0.2541,
//     quantity: 29.2,
//     notional_amount: 7.41972,
//     decision: Short,
//     order_kind: Limit,
//     instrument: Instrument {
//         base: "steem",
//         quote: "usdt",
//     },
// }

// BinanceNewOrderResponseFull {
//     symbol: "STEEMUSDT",
//     order_id: 394523046,
//     order_list_id: -1,
//     client_order_id: "32Pmn--j1PD3YC-UiK8vhRn",
//     transact_time: 1736919552954,
//     price: 0.2541,
//     orig_qty: 29.2,
//     executed_qty: 0.0,
//     cummulative_quote_qty: 0.0,
//     status: New,
//     time_in_force: GTC,
//     type: "LIMIT",
//     side: "SELL",
//     working_time: 1736919552954,
//     fills: [],
// },

// BinanceAccountDataOrder {
//     e: "executionReport",
//     E: 1736919552954,
//     s: "STEEMUSDT",
//     c: "32Pmn--j1PD3YC-UiK8vhRn",
//     S: Sell,
//     o: Limit,
//     f: GTC,
//     q: 29.2,
//     p: 0.2541,
//     P: 0.0,
//     F: 0.0,
//     g: -1,
//     C: "",
//     x: New,
//     X: New,
//     r: "NONE",
//     i: 394523046,
//     l: 0.0,
//     z: 0.0,
//     L: 0.0,
//     n: 0.0,
//     N: None,
//     T: 2025-01-15T05:39:12.954Z,
//     t: -1,
//     v: None,
//     I: 807475335,
//     w: true,
//     m: false,
//     M: false,
//     O: 2025-01-15T05:39:12.954Z,
//     Z: 0.0,
//     Y: 0.0,
//     Q: 0.0,
//     W: 1736919552954,
//     V: "EXPIRE_MAKER",
// },

// BinanceAccountDataBalance {
//     e: "outboundAccountPosition",
//     E: 1736919552954,
//     u: 1736919552954,
//     B: [
//         BinanceAccountDataBalanceVec {
//             a: "BNB",
//             f: 0.0,
//             l: 0.0,
//         },
//         BinanceAccountDataBalanceVec {
//             a: "USDT",
//             f: 5.13170874,
//             l: 0.0,
//         },
//         BinanceAccountDataBalanceVec {
//             a: "STEEM",
//             f: 6.9177,
//             l: 29.2,
//         },
//     ],
// },

// BinanceAccountDataOrder {
//     e: "executionReport",
//     E: 1736919588995,
//     s: "STEEMUSDT",
//     c: "32Pmn--j1PD3YC-UiK8vhRn",
//     S: Sell,
//     o: Limit,
//     f: GTC,
//     q: 29.2,
//     p: 0.2541,
//     P: 0.0,
//     F: 0.0,
//     g: -1,
//     C: "",
//     x: Trade,
//     X: Filled,
//     r: "NONE",
//     i: 394523046,
//     l: 29.2,
//     z: 29.2,
//     L: 0.2541,
//     n: 0.00741972,
//     N: Some(
//         "USDT",
//     ),
//     T: 2025-01-15T05:39:48.995Z,
//     t: 19443630,
//     v: None,
//     I: 807475804,
//     w: false,
//     m: true,
//     M: true,
//     O: 2025-01-15T05:39:12.954Z,
//     Z: 7.41972,
//     Y: 7.41972,
//     Q: 0.0,
//     W: 1736919552954,
//     V: "EXPIRE_MAKER",
// },

// BinanceAccountDataBalance {
//     e: "outboundAccountPosition",
//     E: 1736919588995,
//     u: 1736919588995,
//     B: [
//         BinanceAccountDataBalanceVec {
//             a: "BNB",
//             f: 0.0,
//             l: 0.0,
//         },
//         BinanceAccountDataBalanceVec {
//             a: "USDT",
//             f: 12.54400902,
//             l: 0.0,
//         },
//         BinanceAccountDataBalanceVec {
//             a: "STEEM",
//             f: 6.9177,
//             l: 0.0,
//         },
//     ],
// }

// ##################
// ### Transfered ###
// ##################

// SpotBalanceId(
//     "poloniexspot_op",
// ): Balance {
//     total: 4.2214132,
//     available: 0.0,
// }

// SpotBalanceId(
//     "binancespot_usdt",
// ): Balance {
//     total: 2.57062902,
//     available: 0.0,
// }

// SpotBalanceId(
//     "binancespot_op",
// ): Balance {
//     total: 4.84448,
//     available: 0.0,
// }

// SpotBalanceId(
//     "poloniexspot_usdt",
// ): Balance {
//     total: 4.32465353232,
//     available: 0.0,
// }

// WalletTransfer {
//     trader_id: TraderId(
//         e3fea87e-3362-4973-9d47-d3441b5a5d20,
//     ),
//     coin: "op",
//     wallet_address: "0xc0b2167fc0ff47fe0783ff6e38c0eecc0f784c2f",
//     network: None,
//     amount: 4.27,
// }

// BinanceAccountDataDelta {
//     e: "balanceUpdate",
//     E: 1736963891899,
//     a: "OP",
//     d: -4.27,
//     T: 1736963891898,
// },

// BinanceAccountDataBalance {
//     e: "outboundAccountPosition",
//     E: 1736963891899,
//     u: 1736963891898,
//     B: [
//         BinanceAccountDataBalanceVec {
//             a: "OP",
//             f: 0.57448,
//             l: 0.0,
//         },
//     ],
// },

// BinanceWalletTransferResponse {
//     id: "9297af51a34142f4a0256cb219cda704",
// },

// PoloniexAccountDataBalance {
//     channel: "balances",
//     data: [
//         PoloniexAccountDataBalanceParams {
//             change_time: 1736963941136,
//             account_id: 292264758130978818,
//             account_type: "SPOT",
//             event_type: Deposit,
//             available: 8.4374132,
//             currency: "OP",
//             id: 402642870777061376,
//             user_id: 1887604,
//             hold: 0.0,
//             ts: 1736963941138,
//         },
//     ],
// },
// */
// ////////////////////////////////////////////
// ////////////////////////////////////////////
// ////////////////////////////////////////////
// ////////////////////////////////////////////

// /*----- */
// // Poloniex
// /*----- */
// /*

// ##################
// ### Transfered ###
// ##################

// SpotBalanceId(
//     "binancespot_usdt",
// ): Balance {
//     total: 2.57062902,
//     available: 0.0,
// },
// SpotBalanceId(
//     "poloniexspot_usdt",
// ): Balance {
//     total: 4.32465353232,
//     available: 0.0,
// },
// SpotBalanceId(
//     "binancespot_op",
// ): Balance {
//     total: 0.57448,
//     available: 0.0,
// },
// SpotBalanceId(
//     "poloniexspot_op",
// ): Balance {
//     total: 8.4374132,
//     available: 0.0,
// },

// WalletTransfer {
//     trader_id: TraderId(
//         163dc228-b20f-4f9b-9c00-6506ffa0d1f8,
//     ),
//     coin: "op",
//     wallet_address: "0x1b7c39f6669cee023caff84e06001b03a76f829f",
//     network: None,
//     amount: 3.91,
// }

// PoloniexAccountDataBalance {
//     channel: "balances",
//     data: [
//         PoloniexAccountDataBalanceParams {
//             change_time: 1736964736540,
//             account_id: 292264758130978818,
//             account_type: "SPOT",
//             event_type: Withdraw,
//             available: 4.5274132,
//             currency: "OP",
//             id: 402646206943256577,
//             user_id: 1887604,
//             hold: 3.91,
//             ts: 1736964736542,
//         },
//     ],
// },

// PoloniexWalletTransferResponse {
//     withdrawal_request_id: 18297560,
// },

// PoloniexAccountDataBalance {
//     channel: "balances",
//     data: [
//         PoloniexAccountDataBalanceParams {
//             change_time: 1736964736647,
//             account_id: 292264758130978818,
//             account_type: "SPOT",
//             event_type: Withdraw,
//             available: 4.5274132,
//             currency: "OP",
//             id: 402646207392034816,
//             user_id: 1887604,
//             hold: 0.0,
//             ts: 1736964736651,
//         },
//     ],
// },

// BinanceAccountDataDelta {
//     e: "balanceUpdate",
//     E: 1736964792355,
//     a: "OP",
//     d: 3.61,
//     T: 1736964792354,
// },

// BinanceAccountDataBalance {
//     e: "outboundAccountPosition",
//     E: 1736964792355,
//     u: 1736964792354,
//     B: [
//         BinanceAccountDataBalanceVec {
//             a: "OP",
//             f: 4.18448,
//             l: 0.0,
//         },
//     ],
// },

// ##################
// ### Market Buy ###
// ##################

// SpotBalanceId(
//     "poloniexspot_usdt",
// ): Balance {
//     total: 12.54723142032,
//     available: 0.0,
// },

// OpenOrder {
//     trader_id: TraderId(
//         f4fa85c6-20b8-400d-87d8-d4eac1257f36,
//     ),
//     client_order_id: ClientOrderId(
//         "ul6e5Z1MAe972hFCYe8qyK3",
//     ),
//     price: 10.78,
//     quantity: 0.5,
//     notional_amount: 5.39,
//     decision: Long,
//     order_kind: Market,
//     instrument: Instrument {
//         base: "icp",
//         quote: "usdt",
//     },
// }

// PoloniexAccountDataBalance {
//     channel: "balances",
//     data: [
//         PoloniexAccountDataBalanceParams {
//             change_time: 1736965338382,
//             account_id: 292264758130978818,
//             account_type: "SPOT",
//             event_type: PlaceOrder,
//             available: 7.15723142032,
//             currency: "USDT",
//             id: 402648731251548160,
//             user_id: 1887604,
//             hold: 5.39,
//             ts: 1736965338389,
//         },
//     ],
// },

// PoloniexNewOrderResponse {
//     id: 402648731234775041,
//     client_order_id: "ul6e5Z1MAe972hFCYe8qyK3",
// },

// PoloniexAccountDataOrder {
//     channel: "orders",
//     data: [
//         PoloniexAccountDataOrderParams {
//             symbol: "ICP_USDT",
//             type: Market,
//             quantity: 0.0,
//             order_id: "402648731234775041",
//             trade_fee: 0.0,
//             client_order_id: "ul6e5Z1MAe972hFCYe8qyK3",
//             account_type: "SPOT",
//             fee_currency: "",
//             event_type: Place,
//             source: "API",
//             side: Buy,
//             filled_quantity: 0.0,
//             filled_amount: 0.0,
//             match_role: "",
//             state: New,
//             trade_time: 1970-01-01T00:00:00Z,
//             trade_amount: 0.0,
//             order_amount: 5.39,
//             create_time: 2025-01-15T18:22:18.379Z,
//             price: 0.0,
//             trade_qty: 0.0,
//             trade_price: 0.0,
//             trade_id: 0,
//             ts: 1736965338403,
//         },
//     ],
// },

// PoloniexAccountDataOrder {
//     channel: "orders",
//     data: [
//         PoloniexAccountDataOrderParams {
//             symbol: "ICP_USDT",
//             type: Market,
//             quantity: 0.0,
//             order_id: "402648731234775041",
//             trade_fee: 0.00070042,
//             client_order_id: "ul6e5Z1MAe972hFCYe8qyK3",
//             account_type: "SPOT",
//             fee_currency: "ICP",
//             event_type: Trade,
//             source: "API",
//             side: Buy,
//             filled_quantity: 0.35021,
//             filled_amount: 3.80152955,
//             match_role: "TAKER",
//             state: PartiallyFilled,
//             trade_time: 2025-01-15T18:22:18.395Z,
//             trade_amount: 3.80152955,
//             order_amount: 5.39,
//             create_time: 2025-01-15T18:22:18.379Z,
//             price: 0.0,
//             trade_qty: 0.35021,
//             trade_price: 10.855,
//             trade_id: 55664315,
//             ts: 1736965338433,
//         },
//     ],
// },

// PoloniexAccountDataBalance {
//     channel: "balances",
//     data: [
//         PoloniexAccountDataBalanceParams {
//             change_time: 1736965338432,
//             account_id: 292264758130978818,
//             account_type: "SPOT",
//             event_type: MatchOrder,
//             available: 7.15723142032,
//             currency: "USDT",
//             id: 402648731461251072,
//             user_id: 1887604,
//             hold: 1.58847045,
//             ts: 1736965338436,
//         },
//     ],
// },

// PoloniexAccountDataBalance {
//     channel: "balances",
//     data: [
//         PoloniexAccountDataBalanceParams {
//             change_time: 1736965338432,
//             account_id: 292264758130978818,
//             account_type: "SPOT",
//             event_type: MatchOrder,
//             available: 0.35021,
//             currency: "ICP",
//             id: 402648731461251074,
//             user_id: 1887604,
//             hold: 0.0,
//             ts: 1736965338436,
//         },
//     ],
// },

// PoloniexAccountDataBalance {
//     channel: "balances",
//     data: [
//         PoloniexAccountDataBalanceParams {
//             change_time: 1736965338432,
//             account_id: 292264758130978818,
//             account_type: "SPOT",
//             event_type: MatchOrder,
//             available: 0.34950958,
//             currency: "ICP",
//             id: 402648731461251075,
//             user_id: 1887604,
//             hold: 0.0,
//             ts: 1736965338436,
//         },
//     ],
// },

// PoloniexAccountDataOrder {
//     channel: "orders",
//     data: [
//         PoloniexAccountDataOrderParams {
//             symbol: "ICP_USDT",
//             type: Market,
//             quantity: 0.0,
//             order_id: "402648731234775041",
//             trade_fee: 0.00029206,
//             client_order_id: "ul6e5Z1MAe972hFCYe8qyK3",
//             account_type: "SPOT",
//             fee_currency: "ICP",
//             event_type: Trade,
//             source: "API",
//             side: Buy,
//             filled_quantity: 0.49624,
//             filled_amount: 5.38989786,
//             match_role: "TAKER",
//             state: Filled,
//             trade_time: 2025-01-15T18:22:18.395Z,
//             trade_amount: 1.58836831,
//             order_amount: 5.39,
//             create_time: 2025-01-15T18:22:18.379Z,
//             price: 0.0,
//             trade_qty: 0.14603,
//             trade_price: 10.877,
//             trade_id: 55664316,
//             ts: 1736965338449,
//         },
//     ],
// },

// PoloniexAccountDataBalance {
//     channel: "balances",
//     data: [
//         PoloniexAccountDataBalanceParams {
//             change_time: 1736965338446,
//             account_id: 292264758130978818,
//             account_type: "SPOT",
//             event_type: MatchOrder,
//             available: 7.15723142032,
//             currency: "USDT",
//             id: 402648731519987712,
//             user_id: 1887604,
//             hold: 0.00010214,
//             ts: 1736965338454,
//         },
//     ],
// },

// PoloniexAccountDataBalance {
//     channel: "balances",
//     data: [
//         PoloniexAccountDataBalanceParams {
//             change_time: 1736965338446,
//             account_id: 292264758130978818,
//             account_type: "SPOT",
//             event_type: MatchOrder,
//             available: 7.15733356032,
//             currency: "USDT",
//             id: 402648731519987713,
//             user_id: 1887604,
//             hold: 0.0,
//             ts: 1736965338454,
//         },
//     ],
// },

// PoloniexAccountDataBalance {
//     channel: "balances",
//     data: [
//         PoloniexAccountDataBalanceParams {
//             change_time: 1736965338447,
//             account_id: 292264758130978818,
//             account_type: "SPOT",
//             event_type: MatchOrder,
//             available: 0.49553958,
//             currency: "ICP",
//             id: 402648731524182016,
//             user_id: 1887604,
//             hold: 0.0,
//             ts: 1736965338454,
//         },
//     ],
// },

// PoloniexAccountDataBalance {
//     channel: "balances",
//     data: [
//         PoloniexAccountDataBalanceParams {
//             change_time: 1736965338447,
//             account_id: 292264758130978818,
//             account_type: "SPOT",
//             event_type: MatchOrder,
//             available: 0.49524752,
//             currency: "ICP",
//             id: 402648731524182017,
//             user_id: 1887604,
//             hold: 0.0,
//             ts: 1736965338454,
//         },
//     ],
// },

// ###################
// ### Market sell ###
// ###################

// SpotBalanceId(
//     "poloniexspot_usdt",
// ): Balance {
//     total: 7.15733356032,
//     available: 0.0,
// },

// SpotBalanceId(
//     "poloniexspot_icp",
// ): Balance {
//     total: 0.49524752,
//     available: 0.0,
// },},

// OpenOrder {
//     trader_id: TraderId(
//         2a344d35-8e35-4d62-82d1-2f997e9e29c9,
//     ),
//     client_order_id: ClientOrderId(
//         "32Pv3Wa2pDf2U2ndXSDkCrF",
//     ),
//     price: 10.78,
//     quantity: 0.49,
//     notional_amount: 5.28,
//     decision: Short,
//     order_kind: Market,
//     instrument: Instrument {
//         base: "icp",
//         quote: "usdt",
//     },
// }

// PoloniexNewOrderResponse {
//     id: 402649797208760321,
//     client_order_id: "32Pv3Wa2pDf2U2ndXSDkCrF",
// },

// PoloniexAccountDataBalance {
//     channel: "balances",
//     data: [
//         PoloniexAccountDataBalanceParams {
//             change_time: 1736965592531,
//             account_id: 292264758130978818,
//             account_type: "SPOT",
//             event_type: PlaceOrder,
//             available: 0.00524752,
//             currency: "ICP",
//             id: 402649797229711360,
//             user_id: 1887604,
//             hold: 0.49,
//             ts: 1736965592537,
//         },
//     ],
// },

// PoloniexAccountDataOrder {
//     channel: "orders",
//     data: [
//         PoloniexAccountDataOrderParams {
//             symbol: "ICP_USDT",
//             type: Market,
//             quantity: 0.49,
//             order_id: "402649797208760321",
//             trade_fee: 0.0,
//             client_order_id: "32Pv3Wa2pDf2U2ndXSDkCrF",
//             account_type: "SPOT",
//             fee_currency: "",
//             event_type: Place,
//             source: "API",
//             side: Sell,
//             filled_quantity: 0.0,
//             filled_amount: 0.0,
//             match_role: "",
//             state: New,
//             trade_time: 1970-01-01T00:00:00Z,
//             trade_amount: 0.0,
//             order_amount: 0.0,
//             create_time: 2025-01-15T18:26:32.528Z,
//             price: 0.0,
//             trade_qty: 0.0,
//             trade_price: 0.0,
//             trade_id: 0,
//             ts: 1736965592547,
//         },
//     ],
// },

// PoloniexAccountDataBalance {
//     channel: "balances",
//     data: [
//         PoloniexAccountDataBalanceParams {
//             change_time: 1736965592573,
//             account_id: 292264758130978818,
//             account_type: "SPOT",
//             event_type: MatchOrder,
//             available: 10.91913554032,
//             currency: "USDT",
//             id: 402649797405892608,
//             user_id: 1887604,
//             hold: 0.0,
//             ts: 1736965592577,
//         },
//     ],
// },

// PoloniexAccountDataBalance {
//     channel: "balances",
//     data: [
//         PoloniexAccountDataBalanceParams {
//             change_time: 1736965592573,
//             account_id: 292264758130978818,
//             account_type: "SPOT",
//             event_type: MatchOrder,
//             available: 10.91161193636,
//             currency: "USDT",
//             id: 402649797405892609,
//             user_id: 1887604,
//             hold: 0.0,
//             ts: 1736965592577,
//         },
//     ],
// },

// PoloniexAccountDataBalance {
//     channel: "balances",
//     data: [
//         PoloniexAccountDataBalanceParams {
//             change_time: 1736965592574,
//             account_id: 292264758130978818,
//             account_type: "SPOT",
//             event_type: MatchOrder,
//             available: 0.00524752,
//             currency: "ICP",
//             id: 402649797410086913,
//             user_id: 1887604,
//             hold: 0.13938,
//             ts: 1736965592577,
//         },
//     ],
// },

// PoloniexAccountDataOrder {
//     channel: "orders",
//     data: [
//         PoloniexAccountDataOrderParams {
//             symbol: "ICP_USDT",
//             type: Market,
//             quantity: 0.49,
//             order_id: "402649797208760321",
//             trade_fee: 0.00752360396,
//             client_order_id: "32Pv3Wa2pDf2U2ndXSDkCrF",
//             account_type: "SPOT",
//             fee_currency: "USDT",
//             event_type: Trade,
//             source: "API",
//             side: Sell,
//             filled_quantity: 0.35062,
//             filled_amount: 3.76180198,
//             match_role: "TAKER",
//             state: PartiallyFilled,
//             trade_time: 2025-01-15T18:26:32.541Z,
//             trade_amount: 3.76180198,
//             order_amount: 0.0,
//             create_time: 2025-01-15T18:26:32.528Z,
//             price: 0.0,
//             trade_qty: 0.35062,
//             trade_price: 10.729,
//             trade_id: 55664852,
//             ts: 1736965592587,
//         },
//     ],
// },

// PoloniexAccountDataBalance {
//     channel: "balances",
//     data: [
//         PoloniexAccountDataBalanceParams {
//             change_time: 1736965592587,
//             account_id: 292264758130978818,
//             account_type: "SPOT",
//             event_type: MatchOrder,
//             available: 12.40660181636,
//             currency: "USDT",
//             id: 402649797464588288,
//             user_id: 1887604,
//             hold: 0.0,
//             ts: 1736965592598,
//         },
//     ],
// },

// PoloniexAccountDataBalance {
//     channel: "balances",
//     data: [
//         PoloniexAccountDataBalanceParams {
//             change_time: 1736965592587,
//             account_id: 292264758130978818,
//             account_type: "SPOT",
//             event_type: MatchOrder,
//             available: 12.4036118366,
//             currency: "USDT",
//             id: 402649797464588289,
//             user_id: 1887604,
//             hold: 0.0,
//             ts: 1736965592598,
//         },
//     ],
// },

// PoloniexAccountDataBalance {
//     channel: "balances",
//     data: [
//         PoloniexAccountDataBalanceParams {
//             change_time: 1736965592590,
//             account_id: 292264758130978818,
//             account_type: "SPOT",
//             event_type: MatchOrder,
//             available: 0.00524752,
//             currency: "ICP",
//             id: 402649797477171200,
//             user_id: 1887604,
//             hold: 0.0,
//             ts: 1736965592598,
//         },
//     ],
// },

// PoloniexAccountDataOrder {
//     channel: "orders",
//     data: [
//         PoloniexAccountDataOrderParams {
//             symbol: "ICP_USDT",
//             type: Market,
//             quantity: 0.49,
//             order_id: "402649797208760321",
//             trade_fee: 0.00298997976,
//             client_order_id: "32Pv3Wa2pDf2U2ndXSDkCrF",
//             account_type: "SPOT",
//             fee_currency: "USDT",
//             event_type: Trade,
//             source: "API",
//             side: Sell,
//             filled_quantity: 0.49,
//             filled_amount: 5.25679186,
//             match_role: "TAKER",
//             state: Filled,
//             trade_time: 2025-01-15T18:26:32.541Z,
//             trade_amount: 1.49498988,
//             order_amount: 0.0,
//             create_time: 2025-01-15T18:26:32.528Z,
//             price: 0.0,
//             trade_qty: 0.13938,
//             trade_price: 10.726,
//             trade_id: 55664853,
//             ts: 1736965592603,
//         },
//     ],
// },
// */
