// use arb_bot::exchange_connector::poloniex::poloniex_data;
use arb_bot::exchange_connector::ws::{PingInterval, WebSocketBase, WebSocketPayload};
use futures::StreamExt;
use serde_json::json;

#[tokio::main]
async fn main() {
    // Poloniex
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

    let poloniex_payload = WebSocketPayload {
        url: poloniex_url.to_string(),
        subscription: Some(poloniex_sub),
        ping_interval: Some(poloniex_ping_interval)
    };

//    let mut ws = WebSocketBase::connect(poloniex_payload).await;
//    while let Some(msg) = ws.next().await {
//        println!("{:#?}", msg)
//    }

    // Binance
    let binance_url = "wss://stream.binance.com:9443/stream?streams=btcusdt@depth@100ms/ethusdt@trade";
    let binance_payload = WebSocketPayload {
        url: binance_url.to_string(),
        subscription: None,
        ping_interval: None
    };

    let mut ws = WebSocketBase::connect(binance_payload).await;
    while let Some(msg) = ws.next().await {
        println!("{:#?}", msg)
    }
}

// https://betterprogramming.pub/a-simple-guide-to-using-thiserror-crate-in-rust-eee6e442409b
