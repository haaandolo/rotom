pub struct PoloniexChannel(pub &'static str);

impl PoloniexChannel {
    pub const SPOT_WS_URL: Self = Self("wss://ws.poloniex.com/ws/public");
    pub const TRADES: Self = Self("trades");
    pub const ORDER_BOOK_L2: Self = Self("book_lv2");
}

impl AsRef<str> for PoloniexChannel {
    fn as_ref(&self) -> &str {
        self.0
    }
}