use arb_bot::data::{
    exchange_connector::{binance::BinanceSpot, poloniex::PoloniexSpot},
    subscriber::StreamBuilder,
    StreamType,
};

#[tokio::main]
async fn main() {
    //////////////////
    // Build Streams
    let mut streams = StreamBuilder::new()
        .subscribe([
            (BinanceSpot, "arb", "usdt", StreamType::Trades),
            (BinanceSpot, "arb", "usdt", StreamType::Trades),
            (BinanceSpot, "btc", "usdt", StreamType::L2),
        ])
        // .subscribe([
        //     // (PoloniexSpot, "arb", "usdt", StreamType::Trades),
        //     // (PoloniexSpot, "arb", "usdt", StreamType::Trades),
        //     (PoloniexSpot, "btc", "usdt", StreamType::L2),
        //     (PoloniexSpot, "arb", "usdt", StreamType::L2),
        // ])
        .init()
        .await;

    // Read from socket
    if let Some(mut receiver) = streams.remove("binancespot") {
        while let Some(msg) = receiver.recv().await {
            // Some(msg);
            println!("----- Binance -----");
            println!("{:#?}", msg);
        }
    }

    // Read from socket
    if let Some(mut receiver) = streams.remove("poloniexspot") {
        while let Some(msg) = receiver.recv().await {
            // Some(msg);
            println!("----- Poloniex -----");
            println!("{:#?}", msg);
        }
    }
}

// todo
// - process custom ping for poloniex
// - add mismatch sequence error in websocket
// - multi stream builder
// - make orderbooks