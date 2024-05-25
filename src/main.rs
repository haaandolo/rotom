use arb_bot::exchange_connector::poloniex::poloniex_data;
use arb_bot::exchange_connector::ws::WebSocketBase;
use futures::future::join_all;
use serde_json::json;
use tokio::task;


#[tokio::main]
async fn main() {
    let url = "wss://ws.poloniex.com/ws/public";
    let tickers = vec!["btc_usdt", "arb_usdt"];
    let channels = vec!["book_lv2", "trades"];
    let payload = json!({
        "event": "subscribe",
        "channel": channels,
        "symbols": tickers
    });
    
    let _ = WebSocketBase::connect(url, payload).await;

//    let tickers = vec!["btc_usdt", "arb_usdt"];
//    let channels = vec!["book_lv2", "trades"];
//    let handles = poloniex_data::stream_data(tickers, channels).await;
//    let _ = join_all(handles).await;
}

// https://betterprogramming.pub/a-simple-guide-to-using-thiserror-crate-in-rust-eee6e442409b