use arb_bot::data::{
    protocols::ws::is_websocket_disconnected, subscriber::StreamBuilder, Exchange, StreamType, Sub,
};

#[tokio::main]
async fn main() {
    // Build Streams
    let mut streams = StreamBuilder::new()
        .subscribe(vec![
            Sub::new(Exchange::BinanceSpot, "arb", "usdt", StreamType::Trades),
            Sub::new(Exchange::BinanceSpot, "arb", "usdt", StreamType::Trades),
            //Sub::new(Exchange::BinanceSpot, "btc", "usdt", StreamType::L2),
        ])
        .subscribe(vec![
            Sub::new(Exchange::PoloniexSpot, "arb", "usdt", StreamType::Trades),
            Sub::new(Exchange::PoloniexSpot, "arb", "usdt", StreamType::Trades),
            // Sub::new(Exchange::PoloniexSpot, "btc", "usdt", StreamType::Trades),
        ])
        .init()
        .await;

    // Read from socket
    if let Some(mut value) = streams.clients.remove(&Exchange::PoloniexSpot) {
        let mut stream = value.read_tx.take().unwrap();
        while let Some(msg) = stream.recv().await {
            match msg {
                Ok(_) => (),
                Err(error) => {
                    if is_websocket_disconnected(&error) {
                        println!("{:#?}", error);
                        value.cancel_running_tasks();
                        break;
                    }
                }
            }
            // Some(msg);
            //            println!("----- Poloniex -----");
            //            println!("{:#?}", msg);
        }
    }

    // Read from socket
    if let Some(value) = streams.clients.remove(&Exchange::BinanceSpot) {
        let mut stream = value.read_tx.unwrap();
        while let Some(msg) = stream.recv().await {
            Some(msg);
            //println!("----- Binance -----");
            //println!("{:#?}", msg);
        }
    }
}

// todo
// - serde json to update book
// - ws auto reconnect
// - expected responses for binance spot and poloniex spot
// - write test for the subscribe fn in stream builder
