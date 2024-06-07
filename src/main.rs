use arb_bot::exchange_connector::{
    protocols::ws::is_websocket_disconnected, subscriber::StreamBuilder, Exchange, StreamType, Sub,
};

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

    // Read from socket
    if let Some(mut value) = streams.streams.remove(&Exchange::PoloniexSpot) {
        while let Some(msg) = value.stream.recv().await {
            // match msg {
            //     Ok(_) => (),
            //     Err(error) => {
            //         if is_websocket_disconnected(&error) {
            //             value.cancel_running_tasks();
            //             break;
            //         }
            //     }
            // }
            // Some(msg);
            println!("----- Poloniex -----");
            println!("{:#?}", msg);
        }
    }

    // Read from socket
    if let Some(mut value) = streams.streams.remove(&Exchange::BinanceSpot) {
        while let Some(msg) = value.stream.recv().await {
            // Some(msg);
            println!("----- Binance -----");
            println!("{:#?}", msg);
        }
    }
}

// todo
// - redo builer model for the websocket client
// - ws auto reconnect
// - expected responses for binance spot and poloniex spot
// - write test for the subscribe fn in stream builder
