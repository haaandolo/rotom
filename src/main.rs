use core::str;

use arb_bot::data::{
    exchange::{binance::BinanceSpot, poloniex::PoloniexSpot},
    model::{
        event::{DataKind, MarketEvent},
        event_book::OrderBookL2,
        event_trade::Trades,
        subs::{ExchangeId, StreamKind, StreamType},
    },
    subscriber::{dynamic::DynamicStreams, single::StreamBuilder, Streams},
};

use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    // Initialise logging
    init_logging();

    /*----- */
    // Dynamic streams
    /*----- */
    let mut streams = DynamicStreams::init([
        vec![
            // (ExchangeId::PoloniexSpot, "eth", "usdt", StreamType::L2, StreamKind::Trades),
            // (ExchangeId::PoloniexSpot, "eth", "usdt", StreamType::L2, StreamKind::OrderBookL2),
            // (ExchangeId::BinanceSpot, "sui", "usdt", StreamType::L2, StreamKind::OrderBookL2),
            (ExchangeId::BinanceSpot, "sui", "usdt", StreamType::L2, StreamKind::Trades),
        ],
        vec![
            (ExchangeId::BinanceSpot, "sui", "usdt", StreamType::L2, StreamKind::OrderBookL2),
        //     (ExchangeId::BinanceSpot, "arb", "usdt", StreamType::L2, StreamKind::Trades),
        //     (ExchangeId::PoloniexSpot, "btc", "usdt", StreamType::L2, StreamKind::Trades)
        ],
        vec![
        //     (ExchangeId::BinanceSpot, "ada", "usdt", StreamType::L2, StreamKind::OrderBookL2),
            (ExchangeId::PoloniexSpot, "avax", "usdt", StreamType::L2, StreamKind::Trades)
        ],
        vec![
            (ExchangeId::PoloniexSpot, "avax", "usdt", StreamType::L2, StreamKind::OrderBookL2)
        ],
    ])
    .await
    .unwrap();

    let mut merged = streams.select_all::<MarketEvent<DataKind>>();

    while let Some(event) = merged.next().await {
        println!("{:?}", event)
    }
}
// Initialise an INFO `Subscriber` for `Tracing` Json logs and install it as the global default.
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
// - Dynamic streams
// - Reconnection attempt not working
// - Is websocket disconnect handling
// - DOCUMENTATION + EXAMPLES
// - DOUBLE CHECK TICKER SIZE BEFORE PRODUCTION
// - custom poloniex deserializers
// - process custom ping for poloniex

/*----- */
// Single Streams
/*----- */
// let mut streams = Streams::<OrderBookL2>::builder()
//     .subscribe([
//         (BinanceSpot, "sol", "usdt", StreamType::L2, OrderBookL2),
//         (BinanceSpot, "btc", "usdt", StreamType::L2, OrderBookL2),
//         (BinanceSpot, "btc", "usdt", StreamType::L2, OrderBookL2),
//     ])
//     .subscribe([
//         (PoloniexSpot, "btc", "usdt", StreamType::L2, OrderBookL2),
//         (PoloniexSpot, "arb", "usdt", StreamType::L2, OrderBookL2),
//     ])
//     .init()
//     .await
//     .unwrap();

// // Read from socket
// if let Some(mut receiver) = streams.select(ExchangeId::BinanceSpot) {
//     while let Some(msg) = receiver.recv().await {
//         // Some(msg);
//         println!("----- Binance -----");
//         println!("{:#?}", msg);
//     }
// }

// // Read from socket
// if let Some(mut receiver) = streams.select(ExchangeId::PoloniexSpot) {
//     while let Some(msg) = receiver.recv().await {
//         // Some(msg);
//         println!("----- Poloniex -----");
//         println!("{:#?}", msg);
//     }
// }
