use arb_bot::poloniex_interface::PoloniexInterface;

#[tokio::main]
async fn main() {
    let tickers = vec!["btc_usdt", "arb_usdt"];
    let poloniex_interface = PoloniexInterface::new();
    let _ = poloniex_interface.stream_data(tickers).await;
}
