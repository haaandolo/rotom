use futures::StreamExt;
use rotom_data::{
    event_models::market_event::{DataKind, MarketEvent},
    shared::subscription_models::{ExchangeId, StreamKind},
    streams::builder::dynamic,
};
use rotom_main::{data::live, engine::trader::Trader, strategy::spread::SpreadStategy, };
use tokio::sync::mpsc::{self, UnboundedReceiver};

/*----- */
// Main
/*----- */
#[tokio::main]
pub async fn main() {
    // Initialise logging
    init_logging();

    // Build a trader
    let trader = Trader::builder()
        .data(live::MarketFeed::new(stream_trades().await))
        .strategy(SpreadStategy::default())
        .build()
        .unwrap();

    trader.run()
}

/*----- */
// Setup data feed
/*----- */
async fn stream_trades() -> UnboundedReceiver<MarketEvent<DataKind>> {
    let mut streams = dynamic::DynamicStreams::init([vec![(
        ExchangeId::BinanceSpot,
        "arb",
        "usdt",
        StreamKind::L2,
    ),
    (
        ExchangeId::PoloniexSpot,
        "arb",
        "usdt",
        StreamKind::L2,
    )
    ]])
    .await
    .unwrap();

    let mut data = streams.select_all_l2s();

    let (tx, rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        while let Some(event) = data.next().await {
            let _ = tx.send(MarketEvent::from(event));
        }
    });

    rx
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
