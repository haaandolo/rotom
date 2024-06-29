use arb_bot::data::{subscriber::StreamBuilder, Exchange, StreamType, Sub};

#[tokio::main]
async fn main() {
    //////////////////
    // Build Streams
    let mut streams = StreamBuilder::new()
        .subscribe(vec![
            // Sub::new(Exchange::BinanceSpot, "arb", "usdt", StreamType::Trades),
            // Sub::new(Exchange::BinanceSpot, "arb", "usdt", StreamType::Trades),
            Sub::new(Exchange::BinanceSpot, "btc", "usdt", StreamType::L2),
        ])
        .subscribe(vec![
            // Sub::new(Exchange::PoloniexSpot, "arb", "usdt", StreamType::Trades),
            // Sub::new(Exchange::PoloniexSpot, "arb", "usdt", StreamType::Trades),
            Sub::new(Exchange::PoloniexSpot, "btc", "usdt", StreamType::L2),
        ])
        .init()
        .await;

    // Read from socket
    if let Some(mut receiver) = streams.remove(&Exchange::BinanceSpot) {
        while let Some(msg) = receiver.recv().await {
            // Some(msg);
            println!("----- Binance -----");
            // let msg: BinanceBookUpdate = serde_json::from_str(&msg).unwrap();
            println!("{:#?}", msg);
        }
    }

    // // Read from socket
    // if let Some(mut receiver) = streams.remove(&Exchange::PoloniexSpot) {
    //     while let Some(msg) = receiver.recv().await {
    //         // Some(msg);
    //         println!("----- Poloniex -----");
    //         // let msg: PoloniexBookUpdate = serde_json::from_str(&msg).unwrap();
    //         println!("{:#?}", msg);
    //     }
    // }
}

// todo
// - expected responses and poloniex spot
// - write test for the subscribe fn in stream builder