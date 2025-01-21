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

#[derive(Debug, Deserialize)]
pub struct HtxBookSnaps {
    pub ch: String,
    pub ts: i64,
    pub tick: Tick,
}

#[derive(Debug, Deserialize)]
pub struct Tick {
    #[serde(rename = "seqNum")]
    pub seq_num: i64,
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
}

pub async fn htx_ws_test() {
    let url = "wss://api.huobi.pro/feed";

    let payload = json!({
        "sub": vec![
            // "market.glmrusdt.mbp.5",
            // "market.racausdt.mbp.5",
            "market.glmrusdt.mbp.refresh.5"
        ],
        "id": "id1"
    });

    let (ws_stream, _) = connect_async(url).await.unwrap();
    let (mut write, mut read) = ws_stream.split();

    let _ = write.send(Message::text(payload.to_string())).await;

    let ping_message = PingInterval {
        time: 4,
        message: json!({ "pong": rand::thread_rng().gen::<u64>() }),
    };

    tokio::spawn(schedule_pings_to_exchange(write, ping_message));

    while let Some(msg) = read.next().await {
        if let Message::Binary(binary) = msg.unwrap() {
            let mut decoder = GzDecoder::new(&binary[..]);
            let mut decompressed = String::new();
            match decoder.read_to_string(&mut decompressed) {
                Ok(_) => {
                    println!(
                        "Decompressed: {:?}",
                        decompressed // serde_json::from_str::<HtxBookSnaps>(decompressed.as_str())
                    );
                }
                Err(e) => println!("Error decompressing: {}", e),
            }
        }
    }
}

/*
Decompressed: "{\"id\":\"id1\",\"status\":\"ok\",\"subbed\":\"market.glmrusdt.mbp.refresh.5\",\"ts\":1737439927338}"
Decompressed: "{\"ch\":\"market.glmrusdt.mbp.refresh.5\",\"ts\":1737439924265,\"tick\":{\"seqNum\":534669244,\"bids\":[[0.2236,142.845],[0.2212,244.4487],[0.2211,380.0],[0.221,403.0],[0.2209,4762.5499]],\"asks\":[[0.2248,113.2141],[0.2249,70.0],[0.2291,60.0],[0.2297,169.4166],[0.2298,234.1508]]}}"
Decompressed: "{\"ping\":1737439928888}"
*/
