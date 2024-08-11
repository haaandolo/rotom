use futures::StreamExt;
use rotom_data::{
    event_models::market_event::{DataKind, MarketEvent},
    shared::subscription_models::{ExchangeId, StreamKind},
    streams::builder::dynamic,
};
use rotom_main::{
    data::live,
    engine::trader::Trader,
    execution::{
        simulated::{Config, SimulatedExecution},
        Fees,
    },
    strategy::spread::SpreadStategy,
};
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
        .execution(SimulatedExecution::new(Config {
            simulated_fees_pct: Fees {
                exchange: 0.01,
                slippage: 0.05,
                network: 0.0,
            },
        }))
        .build()
        .unwrap();

    trader.run()
}

/*----- */
// Setup data feed
/*----- */
async fn stream_trades() -> UnboundedReceiver<MarketEvent<DataKind>> {
    let streams = dynamic::DynamicStreams::init([vec![
        // (ExchangeId::BinanceSpot, "op", "usdt", StreamKind::L2),
        (ExchangeId::PoloniexSpot, "op", "usdt", StreamKind::L2),
        // (ExchangeId::BinanceSpot, "op", "usdt", StreamKind::Trades),
        (ExchangeId::PoloniexSpot, "op", "usdt", StreamKind::Trades),
    ]])
    .await
    .unwrap();

    let mut data = streams.select_all::<MarketEvent<DataKind>>();

    let (tx, rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        while let Some(event) = data.next().await {
            println!("{:?}", event);
            let _ = tx.send(event);
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
