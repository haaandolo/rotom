use serde::Deserialize;

/*----- */
// Ticker info
/*----- */
#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct PoloniexSpotTickerInfo {
    symbol: String,
    #[serde(rename = "baseCurrencyName")]
    base_currency_name: String,
    #[serde(rename = "quoteCurrencyName")]
    quote_currency_name: String,
    #[serde(rename = "displayName")]
    display_name: String,
    state: String,
    #[serde(rename = "visibleStartTime")]
    visible_start_time: u64,
    #[serde(rename = "tradableStartTime")]
    tradable_start_time: u64,
    #[serde(rename = "symbolTradeLimit")]
    pub symbol_trade_limit: SymbolTradeLimit,
    #[serde(rename = "crossMargin")]
    cross_margin: CrossMargin,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct SymbolTradeLimit {
    symbol: String,
    #[serde(rename = "priceScale")]
    price_scale: u8,
    #[serde(rename = "quantityScale")]
    pub quantity_scale: u8,
    #[serde(rename = "amountScale")]
    amount_scale: u8,
    #[serde(rename = "minQuantity")]
    min_quantity: String,
    #[serde(rename = "minAmount")]
    min_amount: String,
    #[serde(rename = "highestBid")]
    highest_bid: String,
    #[serde(rename = "lowestAsk")]
    lowest_ask: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct CrossMargin {
    #[serde(rename = "supportCrossMargin")]
    support_cross_margin: bool,
    #[serde(rename = "maxLeverage")]
    max_leverage: u8,
}