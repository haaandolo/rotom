use arb_bot::exchange_connector::poloniex::poloniex_data;


#[tokio::main]
async fn main() {
    let tickers = vec!["btc_usdt", "arb_usdt"];
    let channels = vec!["book_lv2", "trades"];
    let _ = poloniex_data::stream_data(tickers, channels).await;
}

// https://betterprogramming.pub/a-simple-guide-to-using-thiserror-crate-in-rust-eee6e442409b