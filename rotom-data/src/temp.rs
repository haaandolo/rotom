use std::io::Read;

use flate2::read::GzDecoder;
use futures::{SinkExt, StreamExt};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::{
    assets::level::Level,
    protocols::ws::{schedule_pings_to_exchange, PingInterval},
};

pub async fn test_http() {
    let test = reqwest::get("https://api-aws.huobi.pro/v1/settings/common/chains")
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    println!("{:#?}", test);
}

// #[derive(Debug, Deserialize)]
// pub struct HtxBookSnaps {
//     pub ch: String,
//     pub ts: i64,
//     pub tick: Tick,
// }

// #[derive(Debug, Deserialize)]
// pub struct Tick {
//     #[serde(rename = "seqNum")]
//     pub seq_num: i64,
//     pub bids: Vec<Level>,
//     pub asks: Vec<Level>,
// }

// #[derive(Debug, Deserialize)]
// pub struct TradeMarketData {
//     pub ch: String,
//     pub ts: u64,
//     pub tick: Tick2,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct Tick2 {
//     pub id: u64,
//     pub ts: u64,
//     pub data: Vec<Trade2Data>,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct Trade2Data {
//     pub id: u128, // Representing large IDs as `f64` to handle scientific notation
//     pub ts: u64,
//     pub tradeId: u64,
//     pub amount: f64,
//     pub price: f64,
//     pub direction: String,
// }

// pub async fn htx_ws_test() {
//     // let url = "wss://api.huobi.pro/feed";
//     let url = "wss://api-aws.huobi.pro/ws";

//     let payload = json!({
//         "sub": vec![
//             "market.htxusdt.trade.detail",
//             // "market.btcusdt.mbp.refresh.20"
//         ],
//         "id": "id1"
//     });

//     let (ws_stream, _) = connect_async(url).await.unwrap();
//     let (mut write, mut read) = ws_stream.split();

//     let _ = write.send(Message::text(payload.to_string())).await;

//     let ping_message = PingInterval {
//         time: 4,
//         message: json!({ "pong": rand::thread_rng().gen::<u64>() }),
//     };

//     tokio::spawn(schedule_pings_to_exchange(write, ping_message));

//     while let Some(msg) = read.next().await {
//         if let Message::Binary(binary) = msg.unwrap() {
//             let mut decoder = GzDecoder::new(&binary[..]);
//             let mut decompressed = String::new();
//             match decoder.read_to_string(&mut decompressed) {
//                 Ok(_) => {
//                     println!("### Raw ### \n {:#?}", decompressed);
//                     println!(
//                         "### Deserialised ### \n {:?}",
//                         serde_json::from_str::<TradeMarketData>(decompressed.as_str())
//                     );
//                 }
//                 Err(e) => println!("Error decompressing: {}", e),
//             }
//         }
//     }
// }

/*
Decompressed: "{\"id\":\"id1\",\"status\":\"ok\",\"subbed\":\"market.glmrusdt.mbp.refresh.5\",\"ts\":1737439927338}"
Decompressed: "{\"ch\":\"market.glmrusdt.mbp.refresh.5\",\"ts\":1737439924265,\"tick\":{\"seqNum\":534669244,\"bids\":[[0.2236,142.845],[0.2212,244.4487],[0.2211,380.0],[0.221,403.0],[0.2209,4762.5499]],\"asks\":[[0.2248,113.2141],[0.2249,70.0],[0.2291,60.0],[0.2297,169.4166],[0.2298,234.1508]]}}"
Decompressed: "{\"ping\":1737439928888}"
*/
