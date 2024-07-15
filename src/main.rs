use arb_bot::data::{
    exchange::{binance::BinanceSpot, poloniex::PoloniexSpot},
    models::{
        book::OrderBookL2,
        event::{DataKind, MarketEvent},
        subs::{ExchangeId, StreamType},
        trade::Trades,
    },
    subscriber::{single::StreamBuilder, Streams},
};

use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    /*----- */
    // Multi Streams
    /*----- */
    let mut streams: Streams<MarketEvent<DataKind>> = Streams::builder_multi()
        .add(
            Streams::<OrderBookL2>::builder()
                .subscribe([
                    (BinanceSpot, "sol", "usdt", StreamType::L2, OrderBookL2),
                    (BinanceSpot, "btc", "usdt", StreamType::L2, OrderBookL2),
                    (BinanceSpot, "btc", "usdt", StreamType::L2, OrderBookL2),
                ])
                .subscribe([
                    (PoloniexSpot, "sol", "usdt", StreamType::L2, OrderBookL2),
                    (PoloniexSpot, "btc", "usdt", StreamType::L2, OrderBookL2),
                    (PoloniexSpot, "btc", "usdt", StreamType::L2, OrderBookL2),
                ]),
        )
        .add(
            Streams::<Trades>::builder()
                .subscribe([
                    (BinanceSpot, "sol", "usdt", StreamType::Trades, Trades),
                    (BinanceSpot, "btc", "usdt", StreamType::Trades, Trades),
                    (BinanceSpot, "btc", "usdt", StreamType::Trades, Trades),
                ])
                .subscribe([
                    (PoloniexSpot, "sol", "usdt", StreamType::Trades, Trades),
                    (PoloniexSpot, "btc", "usdt", StreamType::Trades, Trades),
                    (PoloniexSpot, "btc", "usdt", StreamType::Trades, Trades),
                ]),
        )
        .init()
        .await
        .unwrap();

    let mut joined_stream = streams.join_map().await;

    for keys in joined_stream.keys() {
        println!("{:#?}", keys)
    }

    while let Some((exchange, data)) = joined_stream.next().await {
        println!(
            "Exchange: {:#?}, MarketEvent<DataKind>: {:#?}",
            exchange, data
        );
    }

    /*----- */
    // Single Streams
    /*----- */
    // let mut streams = Streams::<OrderBookL2>::builder()
    //     .subscribe([
    //         (BinanceSpot, "sol", "usdt", StreamType::L2, OrderBookL2),
    //         (BinanceSpot, "btc", "usdt", StreamType::L2, OrderBookL2),
    //         (BinanceSpot, "btc", "usdt", StreamType::L2, OrderBookL2),
    //     ])
    //     .subscribe([
    //         (PoloniexSpot, "btc", "usdt", StreamType::L2, OrderBookL2),
    //         (PoloniexSpot, "arb", "usdt", StreamType::L2, OrderBookL2),
    //     ])
    //     .init()
    //     .await
    //     .unwrap();

    // // Read from socket
    // if let Some(mut receiver) = streams.select(ExchangeId::BinanceSpot) {
    //     while let Some(msg) = receiver.recv().await {
    //         // Some(msg);
    //         println!("----- Binance -----");
    //         println!("{:#?}", msg);
    //     }
    // }

    // // Read from socket
    // if let Some(mut receiver) = streams.select(ExchangeId::PoloniexSpot) {
    //     while let Some(msg) = receiver.recv().await {
    //         // Some(msg);
    //         println!("----- Poloniex -----");
    //         println!("{:#?}", msg);
    //     }
    // }
}

// todo
// - add mismatch sequence error in websocket
// - make orderbooks
// - make orderbook take in MarketEvent instead of Event
// - process custom ping for poloniex