use std::io::Read;

use crate::{
    exchange::bitstamp::model::BitstampOrderBookSnapshot,
    shared::de::de_str_u64_epoch_ms_as_datetime_utc,
};
use chrono::{DateTime, Utc};
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

/*----- */
// Test Ws
/*----- */
pub async fn test_ws() {
    let url = "wss://ws.bitstamp.net";

    let payload = json!({
        "event": "bts:subscribe",
        "data": {
            "channel": "order_book_btcusdt",
        }
    });

    let (ws_stream, _) = connect_async(url).await.unwrap();
    let (mut write, mut read) = ws_stream.split();

    let _ = write.send(Message::text(payload.to_string())).await;

    // let ping_message = PingInterval {
    //     time: 9,
    //     message: json!({ "event":  "ping"}),
    // };

    // tokio::spawn(schedule_pings_to_exchange(write, ping_message));

    while let Some(msg) = read.next().await {
        // println!("{:?}", msg);
        if let Message::Text(msg) = msg.unwrap() {
            let test = serde_json::from_str::<BitstampOrderBookSnapshot>(msg.as_str());
            println!("{:?}", test);
        }
    }
}

/*
sub errror: Ok(Text("{\"event\":\"bts:error\",\"channel\":\"\",\"data\":{\"code\":null,\"message\":\"Bad subscription string.\"}}"))
sub success:Ok(Text("{\"event\":\"bts:subscription_succeeded\",\"channel\":\"order_book_btcusdt\",\"data\":{}}"))
*/

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
