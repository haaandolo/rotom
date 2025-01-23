use std::io::Read;

use crate::{
    exchange::{
        bitstamp::model::{BitstampOrderBookSnapshot, BitstampSubscriptionResponse, BitstampTrade},
        coinex::model::{CoinExNetworkInfo, CoinExOrderBookSnapshot, CoinExTrade},
    },
    protocols::ws::ws_parser::{StreamParser, WebSocketParser},
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

// "{\"id\":1,\"code\":0,\"message\":\"OK\"}"
// "{\"id\":1,\"code\":20001,\"message\":\"invalid argument\"}"

/*----- */
// Test Ws
/*----- */
pub async fn test_ws() {
    let url = "wss://socket.coinex.com/v2/spot";

    let payload = json!({
      "method": "depth.subscribe",
      "params": {"market_list": [
          ("BTCUSDT", 5, "0", true),
        //   ["ETHUSDT", 10, "0", false]
      ]
      },
      "id": 1
    });

    // let payload = json!({
    //   "method": "deals.subscribe",
    //   "params": {"market_list": ["BTCUSDT"]},
    //   "id": 1
    // });

    let (ws_stream, _) = connect_async(url).await.unwrap();
    let (mut write, mut read) = ws_stream.split();

    let _ = write.send(Message::text(payload.to_string())).await;

    // let ping_message = PingInterval {
    //     time: 9,
    //     message: json!({ "event":  "ping"}),
    // };

    // tokio::spawn(schedule_pings_to_exchange(write, ping_message));

    while let Some(msg) = read.next().await {
        // let test = WebSocketParser::parse::<CoinExTrade>(msg);
        // println!("{:?}", test);

        if let Message::Binary(bin) = msg.unwrap() {
            let mut decoder = GzDecoder::new(&bin[..]);
            let mut decoded = String::new();

            let test = decoder.read_to_string(&mut decoded);
            println!("{:?}", decoded);
        }
    }
}

/*
sub errror: "{\"event\":\"bts:error\",\"channel\":\"\",\"data\":{\"code\":null,\"message\":\"Bad subscription string.\"}}"))
sub success: "{\"event\":\"bts:subscription_succeeded\",\"channel\":\"order_book_btcusdt\",\"data\":{}}"))
*/

/*----- */
// Test http
/*----- */
pub async fn test_http() {
    let test = reqwest::get("https://api.coinex.com/v2/assets/all-deposit-withdraw-config")
        .await
        .unwrap()
        .json::<CoinExNetworkInfo>()
        // .json::<serde_json::Value>()
        .await
        .unwrap();
    println!("{:#?}", test);
}
