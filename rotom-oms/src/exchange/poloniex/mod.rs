pub mod auth;
pub mod poloniex_client;
pub mod requests;
pub mod responses;

use std::{env, error::Error};

use base64::Engine;
use chrono::Utc;
use futures::{SinkExt, StreamExt};
use hmac::{Hmac, Mac};
use rotom_data::{
    protocols::ws::{connect, WsMessage},
    shared::utils::current_timestamp_utc,
};
use serde_json::json;
use sha2::Sha256;

pub async fn poloniex_testing2() -> Result<(), Box<dyn Error>> {
    let poloniex_private_endpoint = "https://api.poloniex.com";

    let timestamp = Utc::now().timestamp_millis();
    let api_secret = env::var("POLONIEX_API_SECRET").expect("Could not find Poloniex Spot secret");
    let api_key = env::var("POLONIEX_API_KEY").expect("Could not find Poloniex Spot API key");
    let query = format!("GET\n/orders\nlimit=5&symbol=OP_USDT");

    let signature = {
        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(api_secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(query.as_bytes());

        let result = mac.finalize();
        let digest = result.into_bytes();

        base64::engine::general_purpose::STANDARD.encode(digest)
    };

    let body = json!({
        "quantity": "5",
        "side": "sell",
        "symbol": "op_usdt"
    });

    let res = reqwest::Client::new()
        .get("https://api.poloniex.com/orders?symbol=OP_USDT&limit=5")
        .header("Content-Type", "application/json")
        .header("key", api_key.as_str())
        .header("signTimestamp", timestamp)
        .header("signature", signature) // TODO
        .send()
        .await?
        .text()
        .await?;

    println!("res ---> {:#?}", res);

    Ok(())
}

pub async fn poloniex_testing() -> Result<(), Box<dyn Error>> {
    let poloniex_private_endpoint = "wss://ws.poloniex.com/ws/private";
    let ws = connect(poloniex_private_endpoint).await?;

    let timestamp = format!("{}", current_timestamp_utc());
    let api_secret = env::var("POLONIEX_API_SECRET").expect("Could not find Poloniex Spot secret");
    let api_key = env::var("POLONIEX_API_KEY").expect("Could not find Poloniex Spot API key");
    let query = format!("{}{}{}{}", "GET\n", "/ws", "\nsignTimestamp=", timestamp);

    let signature = {
        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(api_secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(query.as_bytes());

        let result = mac.finalize();
        let digest = result.into_bytes();

        base64::engine::general_purpose::STANDARD.encode(digest)
    };

    let params = json!(
        {
            "key": api_key.as_str(),
            "signTimestamp": current_timestamp_utc(),
            "signatureMethod": "HmacSHA256",
            "signatureVersion": "2",
            "signature": &signature,
        }
    );

    let res = json!(
        {
            "event": "subscribe",
            "channel": ["auth"],
            "params": params
        }
    );

    let (mut ws_write, mut ws_read) = ws.split();
    let _ = ws_write.send(WsMessage::text(res.to_string())).await;

    while let Some(msg) = ws_read.next().await {
        println!("polo testing {:?}", msg);
    }

    Ok(())
}

//
// POST
// /orders
// requestBody={"symbol": "op_usdt", "side": "sell", "amount": "5"}&signTimestamp=1726972209833
// PSzS6E/UFhbBBe+8qIPy6Hyfl7m/NHR/XADddBFLn10=
// PSzS6E/UFhbBBe+8qIPy6Hyfl7m/NHR/XADddBFLn10=

// PSzS6E/UFhbBBe+8qIPy6Hyfl7m/NHR/XADddBFLn10=
// PSzS6E/UFhbBBe+8qIPy6Hyfl7m/NHR/XADddBFLn10=

// POST
// /orders
// requestBody={"symbol":"op_usdt","side":"sell","amount":"5"}&signTimestamp=1726972209833
