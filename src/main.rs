use arb_bot::exchange_connector::{Exchange, MarketType, StreamType, Subscription};
use arb_bot::exchange_connector::poloniex::PoloniexInterface;
use arb_bot::exchange_connector::binance::BinanceInterface;
use futures::StreamExt;

#[tokio::main]
async fn main() {
//    // BINANCE
//    let mut sub_vec = Vec::new();
//    let sub1 = Subscription::new(Exchange::Binance, "arb".to_string(), "usdt".to_string(), MarketType::Spot, StreamType::L2);
//    let sub2 = Subscription::new(Exchange::Binance, "btc".to_string(), "usdt".to_string(), MarketType::Spot, StreamType::L2);
//    let sub3 = Subscription::new(Exchange::Binance, "arb".to_string(), "usdt".to_string(), MarketType::Spot, StreamType::Trades);

//    sub_vec.push(sub1);
//    sub_vec.push(sub2);
//    sub_vec.push(sub3);

//    let mut bin = BinanceInterface.get_stream(sub_vec).await.unwrap();

//    while let Some(msg) = bin.stream.next().await {
//        println!("{:#?}", msg)
//    }

   //  POLONIEX
    let mut sub_vec = Vec::new();
    let sub1 = Subscription::new(Exchange::Poloniex, "arb".to_string(), "usdt".to_string(), MarketType::Spot, StreamType::L2);
    let sub2 = Subscription::new(Exchange::Poloniex, "btc".to_string(), "usdt".to_string(), MarketType::Spot, StreamType::L2);
    let sub3 = Subscription::new(Exchange::Poloniex, "arb".to_string(), "usdt".to_string(), MarketType::Spot, StreamType::Trades);

    sub_vec.push(sub1);
    sub_vec.push(sub2);
    sub_vec.push(sub3);

    let mut polo = PoloniexInterface.get_stream(sub_vec).await.unwrap();

    while let Some(msg) = polo.stream.next().await {
        println!("{:#?}", msg)
    }

}

// todo
// 1. make a websocket builder for different exchanges and request. look into if you can use builder model
// 2. code expected response for both exchange