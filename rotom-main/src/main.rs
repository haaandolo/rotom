use futures::StreamExt;
use parking_lot::Mutex;
use rotom_data::{
    event_models::market_event::{DataKind, MarketEvent},
    shared::subscription_models::{ExchangeId, Instrument, StreamKind},
    streams::builder::dynamic,
};
use rotom_main::{
    data::{live, Market}, engine::trader::Trader, event::EventTx, execution::{
        simulated::{Config, SimulatedExecution},
        Fees,
    }, oms::{
        allocator::DefaultAllocator, portfolio::MetaPortfolio,
        repository::in_memory::InMemoryRepository, risk::DefaultRisk,
    }, strategy::spread::SpreadStategy
};
use std::sync::Arc;
use tokio::sync::mpsc::{self, UnboundedReceiver};
use uuid::Uuid;

/*----- */
// Main
/*----- */
#[tokio::main]
pub async fn main() {
    // Initialise logging
    init_logging();

    // Engine id
    let engine_id = Uuid::new_v4();

    // Market
    let markets = vec![
        Market::new(ExchangeId::BinanceSpot, Instrument::new("op", "usdt")),
        Market::new(ExchangeId::PoloniexSpot, Instrument::new("op", "usdt")),
    ];

    // Channels
    let (trader_command_tx, trader_command_rx) = mpsc::channel(10);
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let event_tx = EventTx::new(event_tx);

    // Portfolio
    let portfolio = Arc::new(Mutex::new(
        MetaPortfolio::builder()
            .engine_id(engine_id)
            .markets(markets.clone())
            .starting_cash(10000.0)
            .repository(InMemoryRepository::new())
            .allocation_manager(DefaultAllocator {
                default_order_value: 100.0,
            })
            .risk_manager(DefaultRisk {})
            .build_init()
            .unwrap(),
    ));

    // Build trader
    let trader = Trader::builder()
        .engine_id(engine_id)
        .market(markets.clone())
        .command_rx(trader_command_rx)
        .event_tx(event_tx)
        .portfolio(Arc::clone(&portfolio))
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
        (ExchangeId::BinanceSpot, "op", "usdt", StreamKind::L2),
        (ExchangeId::PoloniexSpot, "op", "usdt", StreamKind::L2),
        (ExchangeId::BinanceSpot, "op", "usdt", StreamKind::Trades),
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

/*----- */
// Todo
/*----- */
