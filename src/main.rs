// use arb_bot::exchange_connector::poloniex::poloniex_data;
// use arb_bot::exchange_connector::ws::{PingInterval, WebSocketBase, WebSocketPayload};
// use futures::StreamExt;
// use serde_json::json;

use arb_bot::exchange_connector::poloniex::PoloniexInterface;
use arb_bot::exchange_connector::{Exchange, MarketType, StreamType, Subscription};
use futures::StreamExt;

#[tokio::main]
async fn main() {
    // Binance sub
    let mut subs = Vec::new();
    let binance_sub1 = Subscription {
        exchange: Exchange::Binance,
        base: "arb".to_string(),
        quote: "usdt".to_string(),
        market: MarketType::Spot,
        stream: StreamType::L2
    };
    subs.push(binance_sub1);

    let binance_sub2 = Subscription {
        exchange: Exchange::Binance,
        base: "arb".to_string(),
        quote: "usdt".to_string(),
        market: MarketType::Spot,
        stream: StreamType::Trades
    };
    subs.push(binance_sub2);

    let binance_sub3 = Subscription {
        exchange: Exchange::Binance,
        base: "sui".to_string(),
        quote: "usdt".to_string(),
        market: MarketType::Spot,
        stream: StreamType::Trades
    };
    subs.push(binance_sub3);

    /*-------------------------------------------------- */
    // // Poloniex sub
    // let mut subs = Vec::new();
    // let poloniex_sub1 = Subscription {
    //     exchange: Exchange::Poloniex,
    //     base: "arb".to_string(),
    //     quote: "usdt".to_string(),
    //     market: MarketType::Spot,
    //     stream: StreamType::L2
    // };
    // subs.push(poloniex_sub1);

    // let poloniex_sub2 = Subscription {
    //     exchange: Exchange::Poloniex,
    //     base: "arb".to_string(),
    //     quote: "usdt".to_string(),
    //     market: MarketType::Spot,
    //     stream: StreamType::Trades
    // };
    // subs.push(poloniex_sub2);

    // let poloniex_sub3 = Subscription {
    //     exchange: Exchange::Poloniex,
    //     base: "sui".to_string(),
    //     quote: "usdt".to_string(),
    //     market: MarketType::Spot,
    //     stream: StreamType::Trades
    // };
    // subs.push(poloniex_sub3);

    // let mut polo_stream = PoloniexInterface.get_stream(subs).await.unwrap();

    // while let Some(Ok(msg)) = polo_stream.stream.next().await {
    //     println!("{:#?}", msg)

    // }

    /*-------------------------------------------------------- */
    // Poloniex
    //    let poloniex_url = "wss://ws.poloniex.com/ws/public";
    //    let tickers = vec!["btc_usdt", "arb_usdt"];
    //    let channels = vec!["book_lv2", "trades"];
    //    let poloniex_sub = json!({
    //        "event": "subscribe",
    //        "channel": channels,
    //        "symbols": tickers
    //    });

    //    let poloniex_ping_interval = PingInterval {
    //        time: 20,
    //        message: json!({"event": "ping"}),
    //    };

    //    let poloniex_payload = WebSocketPayload {
    //        url: poloniex_url.to_string(),
    //        subscription: Some(poloniex_sub),
    //        ping_interval: Some(poloniex_ping_interval),
    //    };

    //    let mut ws = WebSocketBase::connect(poloniex_payload).await.unwrap();
    //    while let Some(Ok(msg)) = ws.next().await {
    //        println!("{:#?}", msg)
    //    }

    // Binance
    //    let binance_url = "wss://stream.binance.com:9443/stream?streams=btcusdt@depth@100ms/ethusdt@trade";
    //    let binance_payload = WebSocketPayload {
    //        url: binance_url.to_string(),
    //        subscription: None,
    //        ping_interval: None
    //    };

    //    let mut ws = WebSocketBase::connect(binance_payload).await;
    //    while let Some(msg) = ws.next().await {
    //        println!("{:#?}", msg)
    //    }
}

// https://betterprogramming.pub/a-simple-guide-to-using-thiserror-crate-in-rust-eee6e442409b
