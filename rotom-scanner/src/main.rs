use std::thread;

use actix_web::{web, App, HttpServer};
use rotom_scanner::{
    data::data_streams::get_spot_arb_data_streams,
    server::{
        handlers::{
            get_spread_history_handler, get_top_spreads_handler, get_ws_connection_status_handler,
        },
        server_channels::make_http_channels,
    },
    spot_scanner::scanner::SpotArbScanner,
};
use tokio::sync::Mutex;

/*----- */
// Main
/*----- */
#[tokio::main]
async fn main() -> std::io::Result<()> {
    // async fn main() {
    // Init
    init_logging();

    // Http request for scanner - this has to be abovr market_data_stream for connection fail/success to buffer
    let (scanner_channel, server_channel) = make_http_channels();
    let server_channel = web::Data::new(Mutex::new(server_channel));

    // Init streams
    let (market_data_stream, network_status_stream) = get_spot_arb_data_streams().await;

    // Scanner
    let scanner = SpotArbScanner::new(network_status_stream, market_data_stream, scanner_channel);
    thread::spawn(move || scanner.run());

    // Http server
    HttpServer::new(move || {
        App::new()
            .app_data(server_channel.clone())
            .service(get_top_spreads_handler)
            .service(get_spread_history_handler)
            .service(get_ws_connection_status_handler)
    })
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
