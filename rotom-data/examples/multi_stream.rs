use futures::StreamExt;

use rotom_data::{
    exchange::{binance::BinanceSpotPublicData, poloniex::PoloniexSpotPublicData},
    model::{
        event_book::OrderBookL2,
        event_trade::Trade,
        market_event::{DataKind, MarketEvent},
    },
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
                    (BinanceSpotPublicData, "avax", "usdt", OrderBookL2),
                    (BinanceSpotPublicData, "celo", "usdt", OrderBookL2),
                ])
                .subscribe([
                    (PoloniexSpotPublicData, "naka", "usdt", OrderBookL2),
                    (PoloniexSpotPublicData, "matic", "usdt", OrderBookL2),
                    (PoloniexSpotPublicData, "ada", "usdt", OrderBookL2),
                ]),
        )
        .add(
            Streams::<Trade>::builder()
                .subscribe([
                    (BinanceSpotPublicData, "sol", "usdt", Trade),
                    (BinanceSpotPublicData, "btc", "usdt", Trade),
                    (BinanceSpotPublicData, "btc", "usdt", Trade),
                ])
                .subscribe([
                    (PoloniexSpotPublicData, "sol", "usdt", Trade),
                    (PoloniexSpotPublicData, "btc", "usdt", Trade),
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
