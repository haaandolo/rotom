use std::io::Read;

use crate::{
    exchange::{
        ascendex::model::{
            AscendExBookUpdate, AscendExOrderBookSnapshot, AscendExSubscriptionResponse,
            AscendExTickerInfo,
        },
        bitstamp::model::{BitstampOrderBookSnapshot, BitstampSubscriptionResponse, BitstampTrade},
        coinex::model::{CoinExNetworkInfo, CoinExOrderBookSnapshot, CoinExTrade},
        exmo::model::{ExmoOrderBookSnapshot, ExmoSubscriptionResponse, ExmoTrades},
        kucoin::model::{
            ExmoNetworkInfo, KuCoinNetworkInfo, KuCoinOrderBookSnapshot, KuCoinTrade, KuCoinWsUrl,
        },
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
    let url = "wss://ascendex.com/7/api/pro/v1/stream";

    let payload = json!({
        "op": "sub",
        "id": uuid::Uuid::new_v4(),
        "ch":"depth:ASD/USDT"
    });

    let (ws_stream, _) = connect_async(url).await.unwrap();
    let (mut write, mut read) = ws_stream.split();
    let _ = write.send(Message::text(payload.to_string())).await;

    // let ping_message = PingInterval {
    //     time: 500,
    //     message: json!({ "id":  uuid::Uuid::new_v4(), "type": "ping"}),
    // };

    // request: {"id":63801,"method":"subscribe","topics":["spot/order_book_snapshots:BTC_USDT","spot/order_book_snapshots:ETH_USDT"]}

    // tokio::spawn(schedule_pings_to_exchange(write, ping_message));

    while let Some(msg) = read.next().await {
        // println!("{:?}", msg);
        // println!("###########");

        let test = WebSocketParser::parse::<AscendExBookUpdate>(msg);
        println!("{:?}", test);

        // if let Message::Binary(bin) = msg.unwrap() {
        //     let mut decoder = GzDecoder::new(&bin[..]);
        //     let mut decoded = String::new();

        //     let test = decoder.read_to_string(&mut decoded);
        //     println!("{:?}", decoded);
        // }
    }
}

/*
conn success:
Ok(Text("{\"m\":\"connected\",\"type\":\"unauth\"}"))

succuss:
Ok(Text("{\"m\":\"sub\",\"id\":\"abc123\",\"ch\":\"depth:ASD/USDT\",\"code\":0}"))

error:
Ok(Text("{\"m\":\"error\",\"id\":\"abc123\",\"code\":100005,\"reason\":\"INVALID_WS_REQUEST_DATA\",\"info\":\"Invalid channel: deh:ASD/USDT\"}"))
*/

// /api/v5/asset/currencies
/*----- */
// Test http
/*----- */
pub async fn test_http() {
    //curl --location ''

    // https://ascendex.com/api/pro/v1/depth?symbol=ASD/USDT

    let base_url = "https://ascendex.com";
    // let request_path = "/api/pro/v1/depth";
    let request_path = "/api/pro/v1/cash/products";

    let coin = "ASD/USDT";

    let url = format!("{}{}?symbol={}", base_url, request_path, coin);
    // let url = format!("{}{}", base_url, request_path);
    let test = reqwest::get(url)
        .await
        .unwrap()
        // .text()
        .json::<AscendExTickerInfo>()
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
