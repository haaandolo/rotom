use std::io::Read;

use crate::{
    exchange::{
        bitstamp::model::{BitstampOrderBookSnapshot, BitstampSubscriptionResponse, BitstampTrade},
        coinex::model::{CoinExNetworkInfo, CoinExOrderBookSnapshot, CoinExTrade},
        kucoin::model::{KuCoinNetworkInfo, KuCoinOrderBookSnapshot, KuCoinTrade, KuCoinWsUrl},
        okx::model::{OkxNetworkInfo, OkxOrderBookSnapshot, OkxSubscriptionResponse, OkxTrade},
    },
    protocols::ws::ws_parser::{StreamParser, WebSocketParser},
    shared::de::de_str_u64_epoch_ms_as_datetime_utc,
};
use base64::{engine::general_purpose, Engine};
use chrono::{DateTime, Utc};
use flate2::read::GzDecoder;
use futures::{SinkExt, StreamExt};
use hmac::{Hmac, Mac};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Sha256;
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
    let base_url_http = "https://api.kucoin.com";
    let token_post = "/api/v1/bullet-public";

    let test = reqwest::Client::new()
        .post(format!("{}{}", base_url_http, token_post))
        .send()
        .await
        .unwrap()
        .json::<KuCoinWsUrl>()
        .await
        .unwrap();

    println!("{:#?}", test);

    let url = format!(
        "{}?token={}&[connectId={}]",
        test.data.instance_servers[0].endpoint,
        test.data.token,
        uuid::Uuid::new_v4()
    );

    let payload = json!({
        "id": uuid::Uuid::new_v4(),
        "type": "subscribe",
        // "topic": "/spotMarket/level2Depth5:BTC-USDT,ETH-USDT",
        "topic": "/market/match:BTC-USDT,ETH-USDT",

        "response": true
    });

    let (ws_stream, _) = connect_async(url).await.unwrap();
    let (mut write, mut read) = ws_stream.split();
    let _ = write.send(Message::text(payload.to_string())).await;

    let ping_message = PingInterval {
        time: 30,
        message: json!({ "id":  uuid::Uuid::new_v4(), "type": "ping"}),
    };

    tokio::spawn(schedule_pings_to_exchange(write, ping_message));

    while let Some(msg) = read.next().await {
        // println!("{:?}", msg);
        println!("###########");

        let test = WebSocketParser::parse::<KuCoinTrade>(msg);
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
sub errror: Ok(Text("{\"id\":\"c817503b-341d-473e-af9f-800a9baecc3e\",\"type\":\"error\",\"code\":415,\"data\":\"type is not supported\"}"))
sub success: Ok(Text("{\"id\":\"b1768693-10f3-44ff-a58f-1af4829c6447\",\"type\":\"ack\"}"))

ws conn success: Ok(Text("{\"id\":\"y8rx20rZbc\",\"type\":\"welcome\"}"))
*/

// /api/v5/asset/currencies
/*----- */
// Test http
/*----- */
pub async fn test_http() {
    let base_url = "https://api.kucoin.com";
    let request_path = "/api/v3/currencies";

    let url = format!("{}{}", base_url, request_path);
    let test = reqwest::get(url)
        .await
        .unwrap()
        // .text()
        .json::<KuCoinNetworkInfo>()
        // .json::<serde_json::Value>()
        .await
        .unwrap();
    println!("{:#?}", test);
}

pub async fn test_http_private() {
    let secret = env!("OKX_API_SECRET");
    let key = env!("OKX_API_KEY");
    let passphrase = env!("OKX_PASSPHRASE");

    let timestamp = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let method = "GET";
    let request_path = "/api/v5/asset/currencies";

    let sign_message = format!("{}{}{}", timestamp, method, request_path);
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(sign_message.as_bytes());
    let signature = general_purpose::STANDARD.encode(mac.finalize().into_bytes());

    let test = reqwest::Client::new()
        .get("https://www.okx.com/api/v5/asset/currencies")
        .header("OK-ACCESS-KEY", key)
        .header("OK-ACCESS-SIGN", signature)
        .header("OK-ACCESS-TIMESTAMP", timestamp)
        .header("OK-ACCESS-PASSPHRASE", passphrase)
        .send()
        .await
        .unwrap()
        .json::<OkxNetworkInfo>()
        // .text()
        .await
        .unwrap();

    println!("{:#?}", test);
}

// sign=CryptoJS.enc.Base64.stringify(
//    CryptoJS.HmacSHA256(timestamp + 'GET' + '/api/v5/account/balance?ccy=BTC', SecretKey)
// )
