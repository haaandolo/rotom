use arb_bot::exchange_connector::{subscribe::StreamBuilder, Exchange, StreamType, Sub};
use futures::StreamExt;

#[tokio::main]
async fn main() {
    // Build Streams
    let mut streams = StreamBuilder::new()
        .subscribe(vec![
            Sub::new(Exchange::BinanceSpot, "arb", "usdt", StreamType::Trades),
            Sub::new(Exchange::BinanceSpot, "arb", "usdt", StreamType::Trades),
            Sub::new(Exchange::BinanceSpot, "btc", "usdt", StreamType::Trades),
        ])
        .await
        .subscribe(vec![
            Sub::new(Exchange::PoloniexSpot, "arb", "usdt", StreamType::Trades),
            Sub::new(Exchange::PoloniexSpot, "arb", "usdt", StreamType::Trades),
            Sub::new(Exchange::PoloniexSpot, "btc", "usdt", StreamType::Trades),
        ])
        .await;

//    // Read from socket
//    if let Some(mut value) = streams.streams.remove(&Exchange::PoloniexSpot) {
//        while let Some(msg) = value.stream.next().await {
//            println!("----- Poloniex -----");
//            println!("{:#?}", msg);
//        }
//    }

//    // Read from socket
//    if let Some(mut value) = streams.streams.remove(&Exchange::BinanceSpot) {
//        while let Some(msg) = value.stream.next().await {
//            println!("----- Binance -----");
//            println!("{:#?}", msg);
//        }
//    }
}

// todo
// -3. remove duplicates
// -2. make sure to spawn each ws
// -1. refactor WebsocketBase
// 2. code expected response for both exchange
