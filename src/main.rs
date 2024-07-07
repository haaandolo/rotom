use arb_bot::data::{
    exchange_connector::{binance::{book::BinanceBook, BinanceSpot}, poloniex::{book::PoloniexBookData, PoloniexSpot}}, subscriber::single::StreamBuilder, ExchangeId, StreamType
};

#[tokio::main]
async fn main() {
    //////////////////
    // Build Streams
    let mut streams = StreamBuilder::<BinanceBook>::new()
        .subscribe([
            (BinanceSpot, "arb", "usdt", StreamType::L2, BinanceBook::default()),
            (BinanceSpot, "btc", "usdt", StreamType::L2, BinanceBook::default()),
        ])
    //    .subscribe([
    //        (PoloniexSpot, "btc", "usdt", StreamType::L2, PoloniexBookData::default()),
    //        (PoloniexSpot, "arb", "usdt", StreamType::L2, PoloniexBookData::default()),
    //    ])
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

    // // Read from socket
    // if let Some(mut receiver) = streams.remove(&ExchangeId::PoloniexSpot) {
    //     while let Some(msg) = receiver.recv().await {
    //         // Some(msg);
    //         println!("----- Poloniex -----");
    //         println!("{:#?}", msg);
    //     }
    // }
}

// todo
// - multi stream builder
// - add mismatch sequence error in websocket
// - make orderbooks
// - process custom ping for poloniex
