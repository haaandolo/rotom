use arb_bot::data::{
    exchange::{binance::BinanceSpot, poloniex::PoloniexSpot},
    model::{
        event::{DataKind, MarketEvent},
        event_book::OrderBookL2,
        event_trade::Trades,
        subs::{ExchangeId, StreamType},
    },
    subscriber::{single::StreamBuilder, Streams},
};

use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    // Initialise logging
    init_logging();

    /*----- */
    // Multi Streams
    /*----- */
    let streams: Streams<MarketEvent<DataKind>> = Streams::builder_multi()
        // .add(
        //     Streams::<OrderBookL2>::builder()
        //         .subscribe([
        //             (BinanceSpot, "sol", "usdt", StreamType::L2, OrderBookL2),
        //             (BinanceSpot, "btc", "usdt", StreamType::L2, OrderBookL2),
        //             (BinanceSpot, "eth", "usdt", StreamType::L2, OrderBookL2),
        //             (BinanceSpot, "bnb", "usdt", StreamType::L2, OrderBookL2),
        //             (BinanceSpot, "ada", "usdt", StreamType::L2, OrderBookL2),
        //             (BinanceSpot, "avax", "usdt", StreamType::L2, OrderBookL2),
        //             (BinanceSpot, "celo", "usdt", StreamType::L2, OrderBookL2),
        //         ])
        //         .subscribe([
        //             // (PoloniexSpot, "btc", "usdt", StreamType::L2, OrderBookL2),
        //             // (PoloniexSpot, "eth", "usdt", StreamType::L2, OrderBookL2),
        //             (PoloniexSpot, "sol", "usdt", StreamType::L2, OrderBookL2),
        //             (PoloniexSpot, "arb", "usdt", StreamType::L2, OrderBookL2),
        //             (PoloniexSpot, "sui", "usdt", StreamType::L2, OrderBookL2),
        //             (PoloniexSpot, "trx", "usdt", StreamType::L2, OrderBookL2),
        //             (PoloniexSpot, "naka", "usdt", StreamType::L2, OrderBookL2),
        //             (PoloniexSpot, "matic", "usdt", StreamType::L2, OrderBookL2),
        //             (PoloniexSpot, "ada", "usdt", StreamType::L2, OrderBookL2),
        //         ]),
        // )
        .add(
            Streams::<Trades>::builder()
                // .subscribe([
                //     (BinanceSpot, "sol", "usdt", StreamType::Trades, Trades),
                //     (BinanceSpot, "btc", "usdt", StreamType::Trades, Trades),
                //     (BinanceSpot, "btc", "usdt", StreamType::Trades, Trades),
                // ])
                .subscribe([
                    (PoloniexSpot, "sol", "usdt", StreamType::Trades, Trades),
                    (PoloniexSpot, "btc", "usdt", StreamType::Trades, Trades),
                    (PoloniexSpot, "btc", "usdt", StreamType::Trades, Trades),
                ]),
        )
        .init()
        .await
        .unwrap();

    let mut joined_stream = streams.join_map().await;

    while let Some(data) = joined_stream.next().await {
        println!("@@@@ Market event @@@@");
        println!("{:?}", data);
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
// - how is barter doing exchnage time and received time?
// - process custom ping for poloniex
// - DOCUMENTATION + EXAMPLES
// - DOUBLE CHECK TICKER SIZE BEFORE PRODUCTION
// - Is websocket disconnect handling
// - Reconnection attempt not working
// - Dynamic streams
// - custom poloniex deserializers

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
