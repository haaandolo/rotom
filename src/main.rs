use arb_bot::exchange_connector::{subscribe::StreamBuilder, Exchange, StreamType, Sub};
use futures::StreamExt;

#[tokio::main]
async fn main() {
    // Build Streams
    let mut streams = StreamBuilder::new()
        .subscribe(vec![
            Sub::new(Exchange::BinanceSpot, "arb", "usdt", StreamType::L2),
            Sub::new(Exchange::BinanceSpot, "arb", "usdt", StreamType::Trades),
            Sub::new(Exchange::BinanceSpot, "btc", "usdt", StreamType::L2),
        ])
        .await;

    // Read from socket
    if let Some(mut value) = streams.streams.remove(&Exchange::BinanceSpot) {
        while let Some(msg) = value.stream.next().await {
            println!("{:#?}", msg);
        }
    }
}

// todo
// -1. refactor WebsocketBase
// 1. make a websocket builder for different exchanges and request. look into if you can use builder model
// 2. code expected response for both exchange
