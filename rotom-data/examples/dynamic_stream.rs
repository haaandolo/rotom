use futures::StreamExt;

use rotom_data::{
    model::market_event::{DataKind, MarketEvent},
    shared::subscription_models::{ExchangeId, StreamKind},
    streams::dynamic_stream::DynamicStreams,
};

#[tokio::main]
pub async fn main() {
    // Initialise logging
    init_logging();

    /*----- */
    // Dynamic streams
    /*----- */
    let streams = DynamicStreams::init([
        vec![
            (ExchangeId::PoloniexSpot, "eth", "usdt", StreamKind::Trade),
            (ExchangeId::BinanceSpot, "sui", "usdt", StreamKind::L2),
        ],
        vec![
            (ExchangeId::BinanceSpot, "btc", "usdt", StreamKind::Trade),
            (ExchangeId::PoloniexSpot, "btc", "usdt", StreamKind::Trade),
        ],
        vec![
            (ExchangeId::PoloniexSpot, "ada", "usdt", StreamKind::L2),
            (ExchangeId::PoloniexSpot, "arb", "usdt", StreamKind::L2),
            (ExchangeId::PoloniexSpot, "eth", "usdt", StreamKind::L2),
            (ExchangeId::PoloniexSpot, "btc", "usdt", StreamKind::L2),
        ],
        vec![
            (ExchangeId::BinanceSpot, "arb", "usdt", StreamKind::L2),
            (ExchangeId::BinanceSpot, "eth", "usdt", StreamKind::Trade),
            (ExchangeId::BinanceSpot, "btc", "usdt", StreamKind::Trade),
            (ExchangeId::BinanceSpot, "celo", "usdt", StreamKind::Trade),
        ],
    ])
    .await
    .unwrap();

    let mut merged = streams.select_all::<MarketEvent<DataKind>>();

    while let Some(event) = merged.next().await {
        println!("{:?}", event)
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
