
pub struct BinanceChannel(pub &'static str);

impl BinanceChannel {
    pub const WS_URL: Self = Self("wss://stream.binance.com:9443/stream?streams=");
    pub const TRADES: Self = Self("@trade");
    pub const ORDER_BOOK_L1: Self = Self("@bookTicker");
    pub const ORDER_BOOK_L2: Self = Self("@depth@100ms");
    pub const LIQUIDATIONS: Self = Self("@forceOrder");
}

impl AsRef<str> for BinanceChannel {
    fn as_ref(&self) -> &str {
        self.0
    }
}

pub struct BinanceExchange;

impl BinanceExchange {

}