use arb_bot::data::{
    exchange_connector::{
        binance::{
            book::{BinanceBook, BinanceTrade},
            BinanceSpot,
        },
        poloniex::PoloniexSpot,
    },
    subscriber::StreamBuilder,
    ExchangeId, StreamType,
};

#[tokio::main]
async fn main() {
    //////////////////
    // Build Streams
    let mut streams = StreamBuilder::new()
        .subscribe::<BinanceBook>([
            (BinanceSpot, "arb", "usdt", BinanceBook::default()),
            (BinanceSpot, "btc", "usdt", BinanceBook::default()),
        ])
        .subscribe::<BinanceTrade>([
            (BinanceSpot, "btc", "usdt", BinanceTrade::default()),
            (BinanceSpot, "arb", "usdt", BinanceTrade::default()),
        ])
        // .subscribe::<BinanceTradeUpdate>(vec![
        //     // (PoloniexSpot, "arb", "usdt", StreamType::Trades),
        //     // (PoloniexSpot, "arb", "usdt", StreamType::Trades),
        //     (PoloniexSpot, "btc", "usdt", StreamType::L2),
        //     (PoloniexSpot, "arb", "usdt", StreamType::L2),
        // ])
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
