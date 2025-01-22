use std::io::Read;

use flate2::read::GzDecoder;
use futures::{SinkExt, StreamExt};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::{
    assets::level::Level,
    exchange::woox::model::{WooxNetworkInfo, WooxSubscriptionResponse, WooxTrade},
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

/*----- */
// Test http
/*----- */
pub async fn test_http() {
    let test = reqwest::get("https://api.woox.io/v1/public/token_network")
        .await
        .unwrap()
        .json::<WooxNetworkInfo>()
        .await
        .unwrap();
    println!("{:#?}", test);
}

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
