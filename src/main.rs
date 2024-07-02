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
        // .subscribe([
        //     (BinanceSpot, "arb", "usdt", StreamType::Trades),
        //     (BinanceSpot, "arb", "usdt", StreamType::Trades),
        //     (BinanceSpot, "btc", "usdt", StreamType::L2),
        // ])
        .subscribe([
            // (PoloniexSpot, "arb", "usdt", StreamType::Trades),
            // (PoloniexSpot, "arb", "usdt", StreamType::Trades),
            (PoloniexSpot, "btc", "usdt", StreamType::L2),
            (PoloniexSpot, "arb", "usdt", StreamType::L2),
        ])
        .init()
        .await;

    // // Read from socket
    // if let Some(mut receiver) = streams.remove("binancespot") {
    //     while let Some(msg) = receiver.recv().await {
    //         // Some(msg);
    //         println!("----- Binance -----");
    //         // let msg: BinanceBookUpdate = serde_json::from_str(&msg).unwrap();
    //         println!("{:#?}", msg);
    //     }
    // }

    // Read from socket
    if let Some(mut receiver) = streams.remove("poloniexspot") {
        while let Some(msg) = receiver.recv().await {
            // Some(msg);
            println!("----- Poloniex -----");
            // let msg: PoloniexBookUpdate = serde_json::from_str(&msg).unwrap();
            println!("{:#?}", msg);
        }
    }
}

// todo
// - empty bid ask vec's should be skipped in deserialization process
// - write test for the subscribe fn in stream builder
// - process custom ping for poloniex
// - add mismatch sequence error in websocket
// - multi stream builder
// - make orderbooks