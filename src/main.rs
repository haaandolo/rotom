use arb_bot::data::{
    exchange::{binance::BinanceSpot, poloniex::PoloniexSpot},
    model::{
        event_book::OrderBookL2,
        event::{DataKind, MarketEvent},
        subs::{ExchangeId, StreamType},
        event_trade::Trades,
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
//                .subscribe([
//                    (BinanceSpot, "sol", "usdt", StreamType::L2, OrderBookL2),
//                    (BinanceSpot, "btc", "usdt", StreamType::L2, OrderBookL2),
//                    (BinanceSpot, "eth", "usdt", StreamType::L2, OrderBookL2),
//                    (BinanceSpot, "bnb", "usdt", StreamType::L2, OrderBookL2),
//                    (BinanceSpot, "ada", "usdt", StreamType::L2, OrderBookL2),
//                    (BinanceSpot, "avax", "usdt", StreamType::L2, OrderBookL2),
//                    (BinanceSpot, "celo", "usdt", StreamType::L2, OrderBookL2),
//                ])
                .subscribe([
                    // (PoloniexSpot, "btc", "usdt", StreamType::L2, OrderBookL2),
                    // (PoloniexSpot, "eth", "usdt", StreamType::L2, OrderBookL2),
                    // (PoloniexSpot, "sol", "usdt", StreamType::L2, OrderBookL2),
                    (PoloniexSpot, "arb", "usdt", StreamType::L2, OrderBookL2),
                    (PoloniexSpot, "sui", "usdt", StreamType::L2, OrderBookL2),
                    (PoloniexSpot, "trx", "usdt", StreamType::L2, OrderBookL2),
                    (PoloniexSpot, "naka", "usdt", StreamType::L2, OrderBookL2),
                    (PoloniexSpot, "matic", "usdt", StreamType::L2, OrderBookL2),
                    (PoloniexSpot, "ada", "usdt", StreamType::L2, OrderBookL2),
                ]),
        )
        // .add(
        //     Streams::<Trades>::builder()
        //         .subscribe([
        //             (BinanceSpot, "sol", "usdt", StreamType::Trades, Trades),
        //             (BinanceSpot, "btc", "usdt", StreamType::Trades, Trades),
        //             (BinanceSpot, "btc", "usdt", StreamType::Trades, Trades),
        //         ])
        //         .subscribe([
        //             (PoloniexSpot, "sol", "usdt", StreamType::Trades, Trades),
        //             (PoloniexSpot, "btc", "usdt", StreamType::Trades, Trades),
        //             (PoloniexSpot, "btc", "usdt", StreamType::Trades, Trades),
        //         ]),
        // )
        .init()
        .await
        .unwrap();

    let mut joined_stream = streams.join_map().await;

    while let Some(data) = joined_stream.next().await {
        println!("@@@@ Market event @@@@");
        println!("{:?}", data);
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

/*----- */
// todo
/*----- */
// - rm Some from EventOrderBook
// - remove debugs in traits
// - double sub check
// - are some traits meant to be async traits?
// - process custom ping for poloniex
// - logging
// - custom poloniex deserializers
// - how is barter doing exchnage time and received time?
// - DOCUMENTATION + EXAMPLES
// - DOUBLE CHECK TICKER SIZE BEFORE PRODUCTION
