use arb_bot::data::{
    subscriber::StreamBuilder, Exchange, StreamType, Sub,
};

#[tokio::main]
async fn main() {
    // Build Streams
    let mut streams = StreamBuilder::new()
        .subscribe(vec![
            Sub::new(Exchange::BinanceSpot, "arb", "usdt", StreamType::Trades),
            Sub::new(Exchange::BinanceSpot, "arb", "usdt", StreamType::Trades),
            Sub::new(Exchange::BinanceSpot, "btc", "usdt", StreamType::L2),
        ])
        .subscribe(vec![
            Sub::new(Exchange::PoloniexSpot, "arb", "usdt", StreamType::Trades),
            Sub::new(Exchange::PoloniexSpot, "arb", "usdt", StreamType::Trades),
            Sub::new(Exchange::PoloniexSpot, "btc", "usdt", StreamType::Trades),
        ])
        .init()
        .await;

    // Read from socket
    if let Some(mut receiver) = streams.remove(&Exchange::PoloniexSpot) {
        while let Some(msg) = receiver.recv().await {
            // Some(msg);
            println!("----- Poloniex -----");
            println!("{:#?}", msg);
        }
    }

    // // Read from socket
    // if let Some(mut receiver) = streams.remove(&Exchange::BinanceSpot) {
    //     while let Some(msg) = receiver.recv().await {
    //         // Some(msg);
    //         println!("----- Binance -----");
    //         println!("{:#?}", msg);
    //     }
    // }
}

// todo
// - serde json to update book
// - ws auto reconnect
// - expected responses for binance spot and poloniex spot
// - write test for the subscribe fn in stream builder
