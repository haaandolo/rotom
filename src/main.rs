// use arb_bot::exchange_connector::poloniex::poloniex_data;
use arb_bot::exchange_connector::ws::{PingInterval, WebSocketBase, WebSocketPayload};
use serde_json::json;

#[tokio::main]
async fn main() {
    let poloniex_url = "wss://ws.poloniex.com/ws/public";
    let tickers = vec!["btc_usdt", "arb_usdt"];
    let channels = vec!["book_lv2", "trades"];
    let poloniex_sub = json!({
        "event": "subscribe",
        "channel": channels,
        "symbols": tickers
    });

    let poloniex_ping_interval = PingInterval {
        time: 20,
        message: json!({"event": "ping"})
    };

    // payload with ping 
    let poloniex_payload = WebSocketPayload {
        url: poloniex_url.to_string().clone(),
        subscription: Some(poloniex_sub.clone()),
        ping_interval: Some(poloniex_ping_interval)
    };

    let _ = WebSocketBase::connect(poloniex_payload).await;
}

// https://betterprogramming.pub/a-simple-guide-to-using-thiserror-crate-in-rust-eee6e442409b
