use std::io::Read;

use crate::{
    exchange::{
        bitstamp::model::{BitstampOrderBookSnapshot, BitstampSubscriptionResponse, BitstampTrade},
        coinex::model::{CoinExNetworkInfo, CoinExOrderBookSnapshot, CoinExTrade},
        okx::model::{OkxOrderBookSnapshot, OkxSubscriptionResponse, OkxTrade},
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
    let url = "wss://wspap.okx.com:8443/ws/v5/public";

    let payload = json!({
        "op": "subscribe",
        "args": [
            { "channel": "trades", "instId": "BTC-USDT"},
        ]
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
        // println!("{:?}", msg);
        println!("###########");

        let test = WebSocketParser::parse::<OkxTrade>(msg);
        println!("{:#?}", test);

        // if let Message::Binary(bin) = msg.unwrap() {
        //     let mut decoder = GzDecoder::new(&bin[..]);
        //     let mut decoded = String::new();

        //     let test = decoder.read_to_string(&mut decoded);
        //     println!("{:?}", decoded);
        // }
    }
}

/*
sub errror: "{\"event\":\"subscribe\",\"arg\":{\"channel\":\"books5\",\"instId\":\"BTC-USDT\"},\"connId\":\"0b2ab06e\"}"))
sub success: "{\"event\":\"error\",\"msg\":\"Illegal request: {\\\"args\\\":[{\\\"channel\\\":\\\"books5\\\",\\\"instId\\\":\\\"BTC-USDT\\\"}],\\\"p\\\":\\\"subscribe\\\"}\",\"code\":\"60012\",\"connId\":\"883b44bd\"}"))

Ok(Text("{\"arg\":{\"channel\":\"books5\",\"instId\":\"BTC-USDT\"},\"data\":[{\"asks\":[[\"103170\",\"3.91903957\",\"0\",\"1\"],[\"103171.9\",\"7.39808615\",\"0\",\"1\"],[\"103172.3\",\"5.98890138\",\"0\",\"1\"],[\"103174\",\"5.44041781\",\"0\",\"1\"],[\"103174.4\",\"6.94365576\",\"0\",\"1\"]],\"bids\":[[\"103168.2\",\"0.00345408\",\"0\",\"1\"],[\"103167.9\",\"0.036\",\"0\",\"1\"],[\"103161.4\",\"0.0249803\",\"0\",\"1\"],[\"103161.3\",\"0.02911331\",\"0\",\"1\"],[\"103160.6\",\"0.00193888\",\"0\",\"1\"]],\"instId\":\"BTC-USDT\",\"ts\":\"1737684383905\",\"seqId\":464948033}]}"))
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
