use serde::Deserialize;

use crate::exchange::TickerInfo;

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
    price_scale: usize,
    #[serde(rename = "quantityScale")]
    pub quantity_scale: usize,
    #[serde(rename = "amountScale")]
    amount_scale: usize,
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

impl TickerInfo for PoloniexSpotTickerInfo {
    fn get_asset_price_precision(&self) -> f64 {
        unimplemented!()
    }

    fn get_asset_quantity_precision(&self) -> f64 {
        unimplemented!()
    }
}

/*----- */
// Example
/*----- */
/*
    [
        PoloniexSpotTickerInfo {
            symbol: "OP_USDT",
            base_currency_name: "OP",
            quote_currency_name: "USDT",
            display_name: "OP/USDT",
            state: "NORMAL",
            visible_start_time: 1666940408044,
            tradable_start_time: 1666940408040,
            symbol_trade_limit: SymbolTradeLimit {
                symbol: "OP_USDT",
                price_scale: 4,
                quantity_scale: 4,
                amount_scale: 4,
                min_quantity: "0.0001",
                min_amount: "1",
                highest_bid: "0",
                lowest_ask: "0",
            },
            cross_margin: CrossMargin {
                support_cross_margin: false,
                max_leverage: 1,
            },
        },
    ],
 */