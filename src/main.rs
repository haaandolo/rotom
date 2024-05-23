// use arb_bot::exchange_data::poloniex::poloniex_data;
use serde_json::json;
use arb_bot::exchange_data::ws::{WebSocket, connect};
use tokio_tungstenite::tungstenite::Message;
use futures_util::{sink::SinkExt, StreamExt};

// type ExchangeWsStream<Exchange> = ExchangeStream<WebSocketParser, WebSocket, Exchange>;


#[tokio::main]
async fn main() {
    // let tickers = vec!["btc_usdt", "arb_usdt"];
    // let channels = vec!["book_lv2", "trades"];
    // let _ = poloniex_data::stream_data(tickers, channels).await;

    let url = "wss://ws.poloniex.com/ws/public";
    let tickers = vec!["btc_usdt", "arb_usdt"];
    let channels = vec!["book_lv2", "trades"];
    let payload = json!({
        "event": "subscribe",
        "channel": channels,
        "symbols": tickers
    });
    let mut ws = connect(url).await.unwrap();
    let _ = ws.send(Message::text(payload.to_string())).await;

    while let Some(msg) = ws.next().await {
        let msg = msg.unwrap();
        println!("{:#?}", msg);
    }
}

///////
// use std::time::{Duration, Instant};
// use fast_websocket_client::{client, connect, OpCode};
// use serde_json::json;

// #[tokio::main]
// async fn main() {
//     let url = "wss://ws.poloniex.com/ws/public";
//     let tickers = vec!["btc_usdt", "arb_usdt"];
//     let channels = vec!["book_lv2", "trades"];
//     let payload = json!({
//         "event": "subscribe",
//         "channel": channels,
//         "symbols": tickers
//     });

//     let mut client = connect(url).await.unwrap();
//     client.set_auto_pong(true);
//     client.send_ping(json!({"event": "ping"}).as_str());
//     client.send_json(payload).await.unwrap();

//     loop{
//         let message = if let Ok(result) =
//         tokio::time::timeout(Duration::from_millis(100), client.receive_frame()).await
//         {
//             match result {
//                 Ok(message) => message,
//                 Err(e) => {
//                     eprintln!("Reconnecting from an Error: {e:?}");
//                     let _ = client.send_close(&[]).await;
//                     break; // break the message loop then reconnect
//                 }
//             }
//         } else {
//             println!("timeout");
//             continue;
//         };
//         match message.opcode {
//             OpCode::Text => {
//                 let payload = match simdutf8::basic::from_utf8(message.payload.as_ref()) {
//                     Ok(payload) => payload,
//                     Err(e) => {
//                         eprintln!("Reconnecting from an Error: {e:?}");
//                         let _ = client.send_close(&[]).await;
//                         break; // break the message loop then reconnect
//                     }
//                 };
//                 println!("{payload}");
//             }
//             OpCode::Close => {
//                 println!("{:?}", String::from_utf8_lossy(message.payload.as_ref()));
//                 break
//             }
//             _ => {}
//         }
//     }
// }
