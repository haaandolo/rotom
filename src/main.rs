use arb_bot::data::{
    exchange::{binance::BinanceSpot, poloniex::PoloniexSpot},
    subscriber::Streams,
    models::{book::OrderBookL2, subs::{ExchangeId, StreamType}},
};

#[tokio::main]
async fn main() {
    //////////////////
    // Build Streams
    let mut streams = Streams::<OrderBookL2>::builder()
        .subscribe([
            (BinanceSpot, "arb", "usdt", StreamType::L2, OrderBookL2),
            (BinanceSpot, "btc", "usdt", StreamType::L2, OrderBookL2),
            (BinanceSpot, "btc", "usdt", StreamType::L2, OrderBookL2),
        ])
        .subscribe([
            (PoloniexSpot, "btc", "usdt", StreamType::L2, OrderBookL2),
            (PoloniexSpot, "arb", "usdt", StreamType::L2, OrderBookL2),
        ])
        .init()
        .await;

    // // Read from socket
    // if let Some(mut receiver) = streams.remove(&ExchangeId::BinanceSpot) {
    //     while let Some(msg) = receiver.recv().await {
    //         // Some(msg);
    //         println!("----- Binance -----");
    //         println!("{:#?}", msg);
    //     }
    // }

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
// - process custom ping for poloniex
// - add mismatch sequence error in websocket
// - multi stream builder
// - make orderbooks
// - make orderbook take in MarketEvent instead of Event
