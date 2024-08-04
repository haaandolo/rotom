use arb_bot::data::{
    event_models::{
        event::{DataKind, MarketEvent},
        event_book::OrderBookL2,
        event_trade::Trades,
    },
    exchange::{binance::BinanceSpot, poloniex::PoloniexSpot},
    shared::subscription_models::{ExchangeId, StreamKind},
    streams::builder::{dynamic::DynamicStreams, Streams},
};

use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    // Initialise logging
    init_logging();

    /*----- */
    // Dynamic streams
    /*----- */
    let streams = DynamicStreams::init([
        vec![
            (ExchangeId::PoloniexSpot, "eth", "usdt", StreamKind::Trades),
            // (ExchangeId::PoloniexSpot, "eth", "usdt", StreamKind::L2),
            // (ExchangeId::BinanceSpot, "sui", "usdt", StreamKind::L2),
        ],
        vec![
            (ExchangeId::BinanceSpot, "arb", "usdt", StreamKind::Trades),
            // (ExchangeId::PoloniexSpot, "btc", "usdt", StreamKind::Trades),
        ],
        vec![
            // (ExchangeId::BinanceSpot, "ada", "usdt", StreamKind::L2),
            (ExchangeId::PoloniexSpot, "arb", "usdt", StreamKind::L2),
            // (ExchangeId::BinanceSpot, "celo", "usdt", StreamKind::L2),
        ],
        vec![
            // (ExchangeId::BinanceSpot, "arb", "usdt", StreamKind::Trades),
            // (ExchangeId::BinanceSpot, "eth", "usdt", StreamKind::Trades),
            // (ExchangeId::BinanceSpot, "btc", "usdt", StreamKind::Trades),
            (ExchangeId::BinanceSpot, "btc", "usdt", StreamKind::L2),
        ],
    ])
    .await
    .unwrap();

    let mut merged = streams.select_all::<MarketEvent<DataKind>>();

    while let Some(event) = merged.next().await {
        println!("{:?}", event)
    }

    // /*----- */
    // // Multi Streams
    // /*----- */
    // let streams: Streams<MarketEvent<DataKind>> = Streams::builder_multi()
    //     .add(
    //         Streams::<OrderBookL2>::builder()
    //             .subscribe([
    //                 (BinanceSpot, "avax", "usdt", OrderBookL2),
    //                 (BinanceSpot, "celo", "usdt", OrderBookL2),
    //             ])
    //             .subscribe([
    //                 (PoloniexSpot, "naka", "usdt", OrderBookL2),
    //                 (PoloniexSpot, "matic", "usdt", OrderBookL2),
    //                 (PoloniexSpot, "ada", "usdt", OrderBookL2),
    //             ]),
    //     )
    //     .add(
    //         Streams::<Trades>::builder()
    //             .subscribe([
    //                 (BinanceSpot, "sol", "usdt", Trades),
    //                 (BinanceSpot, "btc", "usdt", Trades),
    //                 (BinanceSpot, "btc", "usdt", Trades),
    //             ])
    //             .subscribe([
    //                 (PoloniexSpot, "sol", "usdt", Trades),
    //                 (PoloniexSpot, "btc", "usdt", Trades),
    //                 (PoloniexSpot, "btc", "usdt", Trades),
    //             ]),
    //     )
    //     .init()
    //     .await
    //     .unwrap();

    // let mut joined_stream = streams.join_map().await;

    // while let Some(data) = joined_stream.next().await {
    //     println!("{:?}", data);
    // }

    // /*----- */
    // // Single Streams
    // /*----- */
    // let mut streams = Streams::<OrderBookL2>::builder()
    //     .subscribe([
    //         (BinanceSpot, "sol", "usdt", OrderBookL2),
    //         (BinanceSpot, "btc", "usdt", OrderBookL2),
    //         (BinanceSpot, "btc", "usdt", OrderBookL2),
    //     ])
    //     .subscribe([
    //         (PoloniexSpot, "btc", "usdt", OrderBookL2),
    //         (PoloniexSpot, "eth", "usdt", OrderBookL2),
    //     ])
    //     .init()
    //     .await
    //     .unwrap();

    // if let Some(mut receiver) = streams.select(ExchangeId::BinanceSpot) {
    //     while let Some(msg) = receiver.recv().await {
    //         println!("----- Binance -----");
    //         println!("{:?}", msg);
    //     }
    // }

    // if let Some(mut receiver) = streams.select(ExchangeId::PoloniexSpot) {
    //     while let Some(msg) = receiver.recv().await {
    //         println!("----- Poloniex -----");
    //         println!("{:?}", msg);
    //     }
    // }
}

/*----- */
// Logging config
/*----- */
fn init_logging() {
    tracing_subscriber::fmt()
        // Filter messages based on the INFO
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
// todo
/*----- */
// - Is websocket disconnect handling
// - DOCUMENTATION + EXAMPLES
// - DOUBLE CHECK TICKER SIZE BEFORE PRODUCTION
// - custom poloniex deserializers
// - process custom ping for poloniex
