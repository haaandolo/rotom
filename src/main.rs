// use arb_bot::exchange_data::poloniex::poloniex_data;
use arb_bot::exchange_data::ws_base::WebSocketClient;
use futures_util::StreamExt;
use serde_json::json;

#[tokio::main]
async fn main() {
    let tickers = vec!["btc_usdt", "arb_usdt"];
    let channels = vec!["book_lv2", "trades"];
    let url = "wss://ws.poloniex.com/ws/public";
    let payload = json!({
        "event": "subscribe",
        "channel": channels,
        "symbols": tickers
    });
    let mut pws = WebSocketClient::new(url.to_string(), payload).await;
    // pws.read_stream().await;
    // pws.set_ping(25).await;

    while let Some(msg) = pws.read_stream.next().await {
        println!("{:#?}", msg.unwrap())
    }
}
