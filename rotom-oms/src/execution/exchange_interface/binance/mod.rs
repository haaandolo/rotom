use std::collections::BTreeMap;
use std::env;
use std::error::Error;

use futures::SinkExt;
use futures::StreamExt;
use hmac::Hmac;
use hmac::Mac;
use rotom_data::protocols::ws::connect;
use rotom_data::protocols::ws::WsMessage;
use rotom_data::shared::utils::current_timestamp_utc;
use serde_json::json;
use sha2::Sha256;

use super::ConnectorPrivate;
use super::HmacSha256;

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
pub struct BinancePrivateWs;

impl ConnectorPrivate for BinancePrivateWs {
    type ApiAuthParams = BinanceAuthParams;

    fn url() -> &'static str {
        BinanceAuthParams::PRIVATE_ENDPOINT
    }

    // fn generate_param_str(order: &OrderEvent) -> String {

    // }

    fn generate_signature(request_str: String) -> String {
        let mut mac = HmacSha256::new_from_slice(BinanceAuthParams::SECRET.as_bytes())
            .expect("Could not generate HMAC for Binance");
        mac.update(request_str.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }
}

/*----- */
// DEL
/*----- */
pub async fn binance_testnet() -> Result<(), Box<dyn Error>> {
    let private_endpoint = "wss://ws-api.binance.com:443/ws-api/v3";
    let ws = connect(private_endpoint).await?;

    let (mut ws_write, mut ws_read) = ws.split();

    let timestamp = format!("{}", current_timestamp_utc());
    let api_key = env::var("BINANCE_API_KEY").expect("Could not get Binance Spot API key");
    let api_secret = env::var("BINANCE_API_SECRET").expect("Could not get Binance Spot secret");

    let mut params = BTreeMap::new();
    params.insert("apiKey", api_key.as_str());
    params.insert("symbol", "BTCUSDT");
    params.insert("side", "BUY");
    params.insert("type", "LIMIT");
    params.insert("price", "50000.0");
    params.insert("quantity", "0.0001");
    params.insert("timeInForce", "GTC");
    params.insert("timestamp", timestamp.as_str());

    let payload = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<String>>()
        .join("&");

    let signature = {
        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(api_secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(payload.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    };

    params.insert("signature", signature.as_str());

    let res = json!(
        {
            "id": "e2a85d9f-07a5-4f94-8d5f-789dc3deb097",
            "method": "order.test",
            "params": params
          }
    );

    let _ = ws_write.send(WsMessage::Text(res.to_string())).await;

    while let Some(msg) = ws_read.next().await {
        println!("testing: {:?}", msg);
    }

    Ok(())
}
