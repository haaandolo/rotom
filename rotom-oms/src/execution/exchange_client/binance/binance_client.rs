use chrono::Utc;
use futures::SinkExt;
use futures::StreamExt;
use hmac::Hmac;
use hmac::Mac;
use rotom_data::protocols::ws::connect;
use rotom_data::protocols::ws::WsMessage;
use rotom_data::shared::subscription_models::Instrument;
use rotom_data::shared::utils::current_timestamp_utc;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use sha2::Sha256;
use std::collections::BTreeMap;
use std::env;
use std::error::Error;

use crate::execution::exchange_client::HmacSha256;
use crate::execution::exchange_client::ParamString;
use crate::execution::exchange_client::PrivateConnector;
use crate::portfolio::OrderEvent;

use super::binance_model::BinanceNewOrder;
use super::binance_model::BinanceNewOrderParams;
use super::binance_model::BinanceSide;
use super::binance_model::BinanceTimeInForce;

/*----- */
// Binance API authentication params
/*----- */
pub struct BinanceAuthParams;

impl BinanceAuthParams {
    pub const SECRET: &'static str = env!("BINANCE_API_SECRET");
    pub const KEY: &'static str = env!("BINANCE_API_KEY");
    pub const PRIVATE_ENDPOINT: &'static str = "wss://ws-api.binance.com:443/ws-api/v3";
}

/*----- */
// Binance Private WebSocket
/*----- */
pub struct BinanceClient;

impl PrivateConnector for BinanceClient {
    type ApiAuthParams = BinanceAuthParams;
    type ExchangeSymbol = BinanceSymbol;

    fn url() -> &'static str {
        BinanceAuthParams::PRIVATE_ENDPOINT
    }

    fn generate_signature(request_str: String) -> String {
        let mut mac = HmacSha256::new_from_slice(BinanceAuthParams::SECRET.as_bytes())
            .expect("Could not generate HMAC for Binance");
        mac.update(request_str.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }
}

fn generate_signature(request_str: String) -> String {
    let mut mac = HmacSha256::new_from_slice(BinanceAuthParams::SECRET.as_bytes())
        .expect("Could not generate HMAC for Binance");
    mac.update(request_str.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

/*----- */
// Binance symbol
/*----- */
pub struct BinanceSymbol(pub String);

impl From<&Instrument> for BinanceSymbol {
    fn from(instrument: &Instrument) -> BinanceSymbol {
        BinanceSymbol(format!("{}{}", instrument.base, instrument.quote).to_uppercase())
    }
}

/*----- */
// DEL
/*----- */
pub async fn binance_generate_order(order_event: OrderEvent) -> Result<(), Box<dyn Error>> {
    let ws = connect(BinanceAuthParams::PRIVATE_ENDPOINT).await?;
    let (mut ws_write, mut ws_read) = ws.split();

    let mut binance_order = BinanceNewOrder::from(order_event);
    let query = ParamString::from(&binance_order);
    let signature = generate_signature(query.0);
    binance_order.params.signature = Some(signature);

    let test = serde_json::to_string(&binance_order)?;

    println!("{:?}", test);

    let _ = ws_write
        .send(WsMessage::Text(serde_json::to_string(&binance_order)?))
        .await;

    while let Some(msg) = ws_read.next().await {
        println!("testing: {:?}", msg);
    }

    Ok(())
}

// pub async fn binance_testnet() -> Result<(), Box<dyn Error>> {
//     let private_endpoint = "wss://ws-api.binance.com:443/ws-api/v3";
//     let ws = connect(private_endpoint).await?;

//     let (mut ws_write, mut ws_read) = ws.split();

//     let timestamp = format!("{}", current_timestamp_utc());
//     let api_key = env::var("BINANCE_API_KEY").expect("Could not get Binance Spot API key");
//     let api_secret = env::var("BINANCE_API_SECRET").expect("Could not get Binance Spot secret");

//     let mut params = BTreeMap::new();
//     params.insert("apiKey", api_key.as_str());
//     params.insert("symbol", "BTCUSDT");
//     params.insert("side", "BUY");
//     params.insert("type", "LIMIT");
//     params.insert("price", "50000.0");
//     params.insert("quantity", "0.0001");
//     params.insert("timeInForce", "GTC");
//     params.insert("timestamp", timestamp.as_str());

//     let payload = params
//         .iter()
//         .map(|(k, v)| format!("{}={}", k, v))
//         .collect::<Vec<String>>()
//         .join("&");

//     println!("{}", payload);

//     let signature = {
//         type HmacSha256 = Hmac<Sha256>;
//         let mut mac = HmacSha256::new_from_slice(api_secret.as_bytes())
//             .expect("HMAC can take key of any size");
//         mac.update(payload.as_bytes());
//         hex::encode(mac.finalize().into_bytes())
//     };

//     params.insert("signature", signature.as_str());

//     let res = json!(
//         {
//             "id": "e2a85d9f-07a5-4f94-8d5f-789dc3deb097",
//             "method": "order.test",
//             "params": params
//           }
//     );

//     let _ = ws_write.send(WsMessage::Text(res.to_string())).await;

//     while let Some(msg) = ws_read.next().await {
//         println!("testing: {:?}", msg);
//     }

//     Ok(())
// }

// apiKey=RoRsfRcLHqT8gUJL3gJefvgQL4mLKYDwzgopBJNorVf5TEprrbXOa4ejyS75lKUZ&price=50000.0&quantity=0.0001&side=BUY&symbol=BTCUSDT&timeInForce=GTC&timestamp=1725444376178&type=LIMIT
// apiKey=RoRsfRcLHqT8gUJL3gJefvgQL4mLKYDwzgopBJNorVf5TEprrbXOa4ejyS75lKUZ&price=50000.0&quantity=0.0001&side=BUY&symbol=BTCUSDT&timeInForce=GTC&timestamp=1725444518756&type=LIMIT
