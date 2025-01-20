use std::io::Read;

use flate2::read::GzDecoder;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[derive(Debug, Deserialize, Serialize)]
struct MbpUpdate {
    ch: String,
    ts: i64,
    tick: Tick,
}

#[derive(Debug, Deserialize, Serialize)]
struct Tick {
    #[serde(rename = "seqNum")]
    seq_num: i64,
    #[serde(rename = "prevSeqNum")]
    prev_seq_num: i64,
    #[serde(default)] // Makes these optional since they're not always present
    asks: Vec<[f64; 2]>,
    #[serde(default)]
    bids: Vec<[f64; 2]>,
}

pub async fn htx_ws_test() {
    let url = "wss://api.huobi.pro/feed";

    let req_payload = json!({
        "req": "market.glmrusdt.mbp.refresh.5",
        "id": "id1",
    });

    let payload = json!({
        "sub": vec![
            "market.glmrusdt.mbp.5",
            // "market.racausdt.mbp.5",
            // "market.glmrusdt.mbp.refresh.5"
        ],
        "id": "id1"
    });

    let (ws_stream, _) = connect_async(url).await.unwrap();
    let (mut write, mut read) = ws_stream.split();

    let _ = write.send(Message::text(payload.to_string())).await;
    let _ = write.send(Message::text(req_payload.to_string())).await;

    while let Some(msg) = read.next().await {
        if let Message::Binary(binary) = msg.unwrap() {
            let mut decoder = GzDecoder::new(&binary[..]);
            let mut decompressed = String::new();
            match decoder.read_to_string(&mut decompressed) {
                Ok(_) => println!(
                    "Decompressed: {:#?}",
                    serde_json::from_str::<MbpUpdate>(decompressed.as_str())
                ),
                Err(e) => println!("Error decompressing: {}", e),
            }
        }
    }
}
