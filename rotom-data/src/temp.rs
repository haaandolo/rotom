use std::io::Read;

use flate2::read::GzDecoder;
use futures::{SinkExt, StreamExt};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::{
    assets::level::Level,
    exchange::woox::model::{WooxSubscriptionResponse, WooxTrade},
    protocols::ws::{schedule_pings_to_exchange, PingInterval},
};

#[derive(Debug, Deserialize)]
pub struct WooxOrderBookSnapshot {
    pub topic: String,
    pub ts: i64,
    pub data: WooxOrderBookSnapshotData,
}

#[derive(Debug, Deserialize)]
pub struct WooxOrderBookSnapshotData {
    pub symbol: String,
    pub asks: Vec<Level>,
    pub bids: Vec<Level>,
}

/*
success: Ok(Text("{\"id\":\"clientid\",\"event\":\"subscribe\",\"success\":true,\"ts\":1737536701623,\"data\":\"SPOT_WOO_USDT@orderbook\"}"))

failed: Ok(Text("{\"id\":\"clientid\",\"event\":\"subscribe\",\"success\":false,\"ts\":1737536796558,\"errorMsg\":\"Exceed the max size of subscribe topics per request.20\"}"))
*/
/*----- */
// Test Ws
/*----- */
pub async fn test_ws() {
    let url = "wss://wss.woox.io/ws/stream/8a6152d8-3f34-42fa-9a23-0aae9fa34208";

    let payload = json!({
        "id": "clientid",
        "topic": "SPOT_ADA_USDT@trade",
        "event": "subscribe",
    });

    let (ws_stream, _) = connect_async(url).await.unwrap();
    let (mut write, mut read) = ws_stream.split();

    let _ = write.send(Message::text(payload.to_string())).await;

    let ping_message = PingInterval {
        time: 9,
        message: json!({ "event":  "ping"}),
    };

    tokio::spawn(schedule_pings_to_exchange(write, ping_message));

    while let Some(msg) = read.next().await {
        // println!("{:?}", msg);
        if let Message::Text(msg) = msg.unwrap() {
            let test = serde_json::from_str::<WooxTrade>(msg.as_str());
            println!("{:?}", test);
        }
    }
}

/*----- */
// Test http
/*----- */
// pub async fn test_http() {
//     let test = reqwest::get("https://api-aws.huobi.pro/v1/settings/common/chains")
//         .await
//         .unwrap()
//         .json::<serde_json::Value>()
//         .await
//         .unwrap();
//     println!("{:#?}", test);
// }
