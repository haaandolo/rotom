use futures::StreamExt;

use rotom_data::{
    event_models::{
        event_book::OrderBookL2,
        event_trade::Trades,
        market_event::{DataKind, MarketEvent},
    },
    exchange::{binance::public::BinanceSpot, poloniex::PoloniexSpot},
    streams::builder::Streams,
};

#[tokio::main]
pub async fn main() {
    // Initialise logging
    init_logging();

    /*----- */
    // Multi Streams
    /*----- */
    let streams: Streams<MarketEvent<DataKind>> = Streams::builder_multi()
        .add(
            Streams::<OrderBookL2>::builder()
                .subscribe([
                    (BinanceSpot, "avax", "usdt", OrderBookL2),
                    (BinanceSpot, "celo", "usdt", OrderBookL2),
                ])
                .subscribe([
                    (PoloniexSpot, "naka", "usdt", OrderBookL2),
                    (PoloniexSpot, "matic", "usdt", OrderBookL2),
                    (PoloniexSpot, "ada", "usdt", OrderBookL2),
                ]),
        )
        .add(
            Streams::<Trades>::builder()
                .subscribe([
                    (BinanceSpot, "sol", "usdt", Trades),
                    (BinanceSpot, "btc", "usdt", Trades),
                    (BinanceSpot, "btc", "usdt", Trades),
                ])
                .subscribe([
                    (PoloniexSpot, "sol", "usdt", Trades),
                    (PoloniexSpot, "btc", "usdt", Trades),
                ]),
        )
        .init()
        .await
        .unwrap();

    let mut joined_stream = streams.join_map().await;

    while let Some(data) = joined_stream.next().await {
        println!("{:?}", data);
    }
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
