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
        .subscribe(vec![
            Sub::new(Exchange::PoloniexSpot, "arb", "usdt", StreamType::Trades),
            Sub::new(Exchange::PoloniexSpot, "arb", "usdt", StreamType::Trades),
            Sub::new(Exchange::PoloniexSpot, "btc", "usdt", StreamType::Trades),
        ])
        .init()
        .await;

    //    // Read from socket
    //    if let Some(mut value) = streams.streams.remove(&Exchange::PoloniexSpot) {
    //        while let Some(msg) = value.stream.next().await {
    //            println!("----- Poloniex -----");
    //            println!("{:#?}", msg);
    //        }
    //    }

       // Read from socket
       if let Some(mut value) = streams.streams.remove(&Exchange::BinanceSpot) {
           while let Some(msg) = value.stream.next().await {
               println!("----- Binance -----");
               println!("{:#?}", msg);
           }
       }
}

// todo
// - ws auto reconnect
// - make sure to spawn each ws
// - refactor code for converting sub to exchange specfic sub
// - expected responses for binance spot and poloniex spot
// - fix the awaits in the stream builder so only have to do it once
// - write test for the subscribe fn in stream builder
