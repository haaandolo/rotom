use actix_web::{App, HttpServer};
use futures::StreamExt;
use rotom_data::{
    exchange::binance::BinanceSpotPublicData,
    model::market_event::{DataKind, MarketEvent},
    shared::subscription_models::{ExchangeId, Instrument, StreamKind},
    streams::dynamic_stream::DynamicStreams,
};
use rotom_scanner::{network_status_stream::NetworkStatusStream, scanner::SpotArbScanner};
use tokio::sync::mpsc;

// #[actix_web::main]
#[tokio::main]
async fn main() -> std::io::Result<()> {
    ///////////
    // Main
    ///////////
    init_logging();

    /////////
    // Playground
    /////////
    // Temp instruments
    let instruments = vec![
        Instrument::new("btc", "usdt"),
        Instrument::new("eth", "usdt"),
        Instrument::new("ada", "usdt"),
        Instrument::new("sol", "usdt"),
        Instrument::new("icp", "usdt"),
    ];

    // Network status stream
    let network = NetworkStatusStream::new()
        // .add_exchange::<AscendExSpotPublicData>(instruments.clone())
        .add_exchange::<BinanceSpotPublicData>(instruments.clone())
        // .add_exchange::<ExmoSpotPublicData>(instruments.clone())
        // .add_exchange::<HtxSpotPublicData>(instruments.clone())
        // .add_exchange::<KuCoinSpotPublicData>(instruments.clone())
        // .add_exchange::<OkxSpotPublicData>(instruments.clone())
        // .add_exchange::<WooxSpotPublicData>(instruments.clone())
        .build();

    // Market data feed
    let streams = DynamicStreams::init([vec![
        (ExchangeId::BinanceSpot, "ada", "usdt", StreamKind::Trade),
        (ExchangeId::BinanceSpot, "ada", "usdt", StreamKind::L2),
        (ExchangeId::BinanceSpot, "icp", "usdt", StreamKind::Trade),
        (ExchangeId::BinanceSpot, "icp", "usdt", StreamKind::L2),
        (ExchangeId::BinanceSpot, "sol", "usdt", StreamKind::Trade),
        (ExchangeId::BinanceSpot, "sol", "usdt", StreamKind::L2),
    ]])
    .await
    .unwrap();

    let mut data = streams.select_all::<MarketEvent<DataKind>>();
    let (market_data_tx, market_data_rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        while let Some(event) = data.next().await {
            println!("{:?}", event);
            let _ = market_data_tx.send(event);
        }
    });

    // Http request for scanner
    let (http_request_tx, http_request_rx) = mpsc::channel(10);
    let (http_response_tx, http_response_rx) = mpsc::channel(10);

    // Scanner
    let scanner = SpotArbScanner::new(network, market_data_rx, http_request_rx, http_response_tx);

    // Spawn scanner
    tokio::spawn(async move { scanner.run().await });

    // Http server
    HttpServer::new(|| App::new())
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
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
