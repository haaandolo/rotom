// use std::collections::{HashMap, HashSet};

// use arb_bot::data::{
//     exchange_connector::{binance::BinanceSpot, poloniex::PoloniexSpot, Connector},
//     ConnectorStuct, ExchangeSub, StreamType,
// };

// fn subscribe<'a, Subscriptions, Exchange>(subscriptions: Subscriptions)
// where
//     Subscriptions: IntoIterator<Item = (Exchange, &'a str, &'a str, StreamType)>,
//     Exchange: Connector + std::cmp::Eq + std::hash::Hash + std::fmt::Debug,
// {
//     // exchange_sub is a hashmap of where the key is a exchange connector and
//     // value is a vec of ExchangeSub structs. Here we are essentially grouping
//     // by the exchange connector and geting the subscriptions associated with it
//     let mut exchange_sub = HashMap::new();
//     subscriptions
//         .into_iter()
//         .collect::<HashSet<_>>() // rm duplicates
//         .into_iter()
//         .for_each(|(exchange_conn, quote, base, stream_type)| {
//             exchange_sub
//                 .entry(exchange_conn)
//                 .or_insert(Vec::new())
//                 .push(ExchangeSub::new(base, quote, stream_type))
//         });

//     // Here we move the key of exchange_sub into a new stuct and replace it 
//     // with a string
//     let mut some = exchange_sub
//         .into_iter()
//         .map(|(key, value)| {
//             let exchange_name = key.exchange_id();
//             let conn_struct = ConnectorStuct {
//                 connector: key,
//                 subs: value,
//             };
//             (exchange_name, conn_struct)
//         })
//         .collect::<HashMap<_, _>>();

//     let test = some.remove("binancespot").unwrap();

//     println!("{:#?}", test.connector.url());
// }

// #[tokio::main]
// async fn main() {
//     let subs = [
//         (BinanceSpot, "arb", "usdt", StreamType::Trades),
//         (BinanceSpot, "arb", "usdt", StreamType::Trades),
//         (BinanceSpot, "btc", "usdt", StreamType::L2),
//     ];

//     let _ = subscribe(subs);
// }

use arb_bot::data::{exchange_connector::{binance::BinanceSpot, poloniex::PoloniexSpot}, subscriber::StreamBuilder, ExchangeId, StreamType, Sub};

#[tokio::main]
async fn main() {
    //////////////////
    // Build Streams
    let mut streams = StreamBuilder::new()
        .subscribe([
            (BinanceSpot, "arb", "usdt", StreamType::Trades),
            (BinanceSpot, "arb", "usdt", StreamType::Trades),
            (BinanceSpot, "btc", "usdt", StreamType::L2),
        ]);
        // .await;

    // // Read from socket
    // if let Some(mut receiver) = streams.remove(&Exchange::BinanceSpot) {
    //     while let Some(msg) = receiver.recv().await {
    //         // Some(msg);
    //         println!("----- Binance -----");
    //         // let msg: BinanceBookUpdate = serde_json::from_str(&msg).unwrap();
    //         println!("{:#?}", msg);
    //     }
    // }

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
// - empty bid ask vec's should be skipped in deserialization process
