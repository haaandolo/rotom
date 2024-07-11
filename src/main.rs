use arb_bot::data::{
    exchange_connector::{binance::BinanceSpot, poloniex::PoloniexSpot},
    subscriber::Streams,
    ExchangeId, OrderbookL2, StreamType,
};

#[tokio::main]
async fn main() {
    //////////////////
    // Build Streams
    let mut streams = Streams::<OrderbookL2>::builder()
        .subscribe([
            (BinanceSpot, "arb", "usdt", StreamType::L2, OrderbookL2),
            (BinanceSpot, "btc", "usdt", StreamType::L2, OrderbookL2),
            (BinanceSpot, "btc", "usdt", StreamType::L2, OrderbookL2),
        ])
        .subscribe([
            (PoloniexSpot, "btc", "usdt", StreamType::L2, OrderbookL2),
            (PoloniexSpot, "arb", "usdt", StreamType::L2, OrderbookL2),
        ])
        .init()
        .await;

    // Read from socket
    if let Some(mut receiver) = streams.remove(&ExchangeId::BinanceSpot) {
        while let Some(msg) = receiver.recv().await {
            // Some(msg);
            println!("----- Binance -----");
            println!("{:#?}", msg);
        }
    }

    // Read from socket
    if let Some(mut receiver) = streams.remove(&ExchangeId::PoloniexSpot) {
        while let Some(msg) = receiver.recv().await {
            // Some(msg);
            println!("----- Poloniex -----");
            println!("{:#?}", msg);
        }
    }
}

// todo
// - impl sub kinds for events
// - process custom ping for poloniex
// - add mismatch sequence error in websocket
// - multi stream builder
// - make orderbooks
