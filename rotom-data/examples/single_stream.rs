use rotom_data::{
    model::event_book::OrderBookL2,
    exchange::{binance::public_ws_stream::BinanceSpot, poloniex::public_ws_stream::PoloniexSpot},
    shared::subscription_models::ExchangeId,
    streams::builder::Streams,
};

#[tokio::main]
pub async fn main() {
    // Initialise logging
    init_logging();

    /*----- */
    // Single Streams
    /*----- */
    let mut streams = Streams::<OrderBookL2>::builder()
        .subscribe([
            (BinanceSpot, "sol", "usdt", OrderBookL2),
            (BinanceSpot, "btc", "usdt", OrderBookL2),
            (BinanceSpot, "arb", "usdt", OrderBookL2),
        ])
        .subscribe([
            (PoloniexSpot, "btc", "usdt", OrderBookL2),
            (PoloniexSpot, "eth", "usdt", OrderBookL2),
        ])
        .init()
        .await
        .unwrap();

    if let Some(mut receiver) = streams.select(ExchangeId::BinanceSpot) {
        while let Some(msg) = receiver.recv().await {
            println!("----- Binance -----");
            println!("{:?}", msg);
        }
    }

    if let Some(mut receiver) = streams.select(ExchangeId::PoloniexSpot) {
        while let Some(msg) = receiver.recv().await {
            println!("----- Poloniex -----");
            println!("{:?}", msg);
        }
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
