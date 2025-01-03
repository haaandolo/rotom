use serde::Deserialize;

use crate::{
    model::ticker_info::{TickerInfo, TickerPrecision},
    shared::de::de_str,
};

/*----- */
// Ticker info
/*----- */
// Reference: https://developers.binance.com/docs/binance-spot-api-docs/rest-api/general-endpoints#exchange-information
#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct BinanceSpotTickerInfo {
    timezone: String,
    #[serde(rename = "serverTime")]
    server_time: u64,
    #[serde(rename = "rateLimits")]
    rate_limits: Vec<RateLimit>,
    #[serde(rename = "exchangeFilters")]
    exchange_filters: Vec<serde_json::Value>,
    pub symbols: Vec<Ticker>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct RateLimit {
    #[serde(rename = "rateLimitType")]
    rate_limit_type: String,
    interval: String,
    #[serde(rename = "intervalNum")]
    interval_num: u32,
    limit: u32,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct Ticker {
    symbol: String,
    status: String,
    #[serde(rename = "baseAsset")]
    base_asset: String,
    #[serde(rename = "baseAssetPrecision")]
    base_asset_precision: u8,
    #[serde(rename = "quoteAsset")]
    quote_asset: String,
    #[serde(rename = "quotePrecision")]
    quote_precision: u8,
    #[serde(rename = "quoteAssetPrecision")]
    quote_asset_precision: u8,
    #[serde(rename = "baseCommissionPrecision")]
    base_commission_precision: u8,
    #[serde(rename = "quoteCommissionPrecision")]
    quote_commission_precision: u8,
    #[serde(rename = "orderTypes")]
    order_types: Vec<String>,
    #[serde(rename = "icebergAllowed")]
    iceberg_allowed: bool,
    #[serde(rename = "ocoAllowed")]
    oco_allowed: bool,
    #[serde(rename = "quoteOrderQtyMarketAllowed")]
    quote_order_qty_market_allowed: bool,
    #[serde(rename = "allowTrailingStop")]
    allow_trailing_stop: bool,
    #[serde(rename = "cancelReplaceAllowed")]
    cancel_replace_allowed: bool,
    #[serde(rename = "isSpotTradingAllowed")]
    is_spot_trading_allowed: bool,
    #[serde(rename = "isMarginTradingAllowed")]
    is_margin_trading_allowed: bool,
    pub filters: Vec<Filter>,
    permissions: Vec<String>,
    #[serde(rename = "defaultSelfTradePreventionMode")]
    default_self_trade_prevention_mode: String,
    #[serde(rename = "allowedSelfTradePreventionModes")]
    allowed_self_trade_prevention_modes: Vec<String>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "filterType")]
pub enum Filter {
    #[serde(rename = "PRICE_FILTER")]
    PriceFilter {
        #[serde(rename = "minPrice", deserialize_with = "de_str")]
        min_price: f64,
        #[serde(rename = "maxPrice", deserialize_with = "de_str")]
        max_price: f64,
        #[serde(rename = "tickSize", deserialize_with = "de_str")]
        tick_size: f64, // This value represents the price a given asset can +ve or -ve by, hence is the quote asset tick value
    },
    #[serde(rename = "PERCENT_PRICE")]
    PercentPrice {
        #[serde(rename = "multiplierUp")]
        multiplier_up: String,
        #[serde(rename = "multiplierDown")]
        multiplier_down: String,
        #[serde(rename = "avgPriceMins")]
        avg_price_mins: u32,
    },
    #[serde(rename = "LOT_SIZE")]
    LotSize {
        #[serde(rename = "minQty", deserialize_with = "de_str")]
        min_qty: f64,
        #[serde(rename = "maxQty", deserialize_with = "de_str")]
        max_qty: f64,
        #[serde(rename = "stepSize", deserialize_with = "de_str")]
        step_size: f64, // This value represents the quantity a given asset can +ve or -ve by, hence is the base asset tick value
    },
    #[serde(rename = "MIN_NOTIONAL")]
    MinNotional {
        #[serde(rename = "minNotional")]
        min_notional: String,
        #[serde(rename = "applyToMarket")]
        apply_to_market: bool,
        #[serde(rename = "avgPriceMins")]
        avg_price_mins: u32,
    },
    #[serde(rename = "ICEBERG_PARTS")]
    IcebergParts { limit: u32 },
    #[serde(rename = "MARKET_LOT_SIZE")]
    MarketLotSize {
        #[serde(rename = "minQty")]
        min_qty: String,
        #[serde(rename = "maxQty")]
        max_qty: String,
        #[serde(rename = "stepSize")]
        step_size: String,
    },
    #[serde(rename = "TRAILING_DELTA")]
    TrailingDelta {
        #[serde(rename = "minTrailingAboveDelta")]
        min_trailing_above_delta: u32,
        #[serde(rename = "maxTrailingAboveDelta")]
        max_trailing_above_delta: u32,
        #[serde(rename = "minTrailingBelowDelta")]
        min_trailing_below_delta: u32,
        #[serde(rename = "maxTrailingBelowDelta")]
        max_trailing_below_delta: u32,
    },
    #[serde(rename = "PERCENT_PRICE_BY_SIDE")]
    PercentPriceBySide {
        #[serde(rename = "bidMultiplierUp")]
        bid_multiplier_up: String,
        #[serde(rename = "bidMultiplierDown")]
        bid_multiplier_down: String,
        #[serde(rename = "askMultiplierUp")]
        ask_multiplier_up: String,
        #[serde(rename = "askMultiplierDown")]
        ask_multiplier_down: String,
        #[serde(rename = "avgPriceMins")]
        avg_price_mins: u32,
    },
    #[serde(rename = "MAX_NUM_ORDERS")]
    MaxNumOrders {
        #[serde(rename = "maxNumOrders")]
        max_num_orders: u32,
    },
    #[serde(rename = "MAX_NUM_ALGO_ORDERS")]
    MaxNumAlgoOrders {
        #[serde(rename = "maxNumAlgoOrders")]
        max_num_algo_orders: u32,
    },
}

impl From<BinanceSpotTickerInfo> for TickerInfo {
    fn from(info: BinanceSpotTickerInfo) -> Self {
        let price_precision = info.symbols[0].filters.iter().find_map(|filter| {
            if let Filter::PriceFilter { tick_size, .. } = filter {
                Some(tick_size.to_owned())
            } else {
                None
            }
        }).unwrap_or_else(|| panic!("Price precision for Binance should never panic. This failed for this ticker: {:#?}", info));

        let quantity_precision = info.symbols[0].filters.iter().find_map(|filter| {
            if let Filter::LotSize { step_size, .. } = filter {
                Some(step_size.to_owned())
            } else {
                None
            }
        }).unwrap_or_else(|| panic!("Quantity precision for Binance should never panic. This failed for this ticker: {:#?}", info));

        Self {
            precision: TickerPrecision {
                price_precision,
                quantity_precision,
                notional_precision: price_precision,
            },
        }
    }
}

/*----- */
// Example
/*----- */
/*
    BinanceSpotTickerInfo {
        timezone: "UTC",
        server_time: 1735846005380,
        rate_limits: [
            RateLimit {
                rate_limit_type: "REQUEST_WEIGHT",
                interval: "MINUTE",
                interval_num: 1,
                limit: 1200,
            },
            RateLimit {
                rate_limit_type: "ORDERS",
                interval: "SECOND",
                interval_num: 10,
                limit: 100,
            },
            RateLimit {
                rate_limit_type: "ORDERS",
                interval: "DAY",
                interval_num: 1,
                limit: 200000,
            },
            RateLimit {
                rate_limit_type: "RAW_REQUESTS",
                interval: "MINUTE",
                interval_num: 5,
                limit: 6100,
            },
        ],
        exchange_filters: [],
        symbols: [
            Ticker {
                symbol: "OPUSDT",
                status: "TRADING",
                base_asset: "OP",
                base_asset_precision: 8,
                quote_asset: "USDT",
                quote_precision: 8,
                quote_asset_precision: 8,
                base_commission_precision: 8,
                quote_commission_precision: 8,
                order_types: [
                    "LIMIT",
                    "LIMIT_MAKER",
                    "MARKET",
                    "STOP_LOSS_LIMIT",
                    "TAKE_PROFIT_LIMIT",
                ],
                iceberg_allowed: true,
                oco_allowed: true,
                quote_order_qty_market_allowed: true,
                allow_trailing_stop: true,
                cancel_replace_allowed: true,
                is_spot_trading_allowed: true,
                is_margin_trading_allowed: false,
                filters: [
                    PriceFilter {
                        min_price: 0.001,
                        max_price: 1000.0,
                        tick_size: 0.001,
                    },
                    PercentPrice {
                        multiplier_up: "5",
                        multiplier_down: "0.2",
                        avg_price_mins: 5,
                    },
                    LotSize {
                        min_qty: 0.01,
                        max_qty: 92141578.0,
                        step_size: 0.01,
                    },
                    MinNotional {
                        min_notional: "1.00000000",
                        apply_to_market: true,
                        avg_price_mins: 5,
                    },
                    IcebergParts {
                        limit: 10,
                    },
                    MarketLotSize {
                        min_qty: "0.00000000",
                        max_qty: "6840.55832635",
                        step_size: "0.00000000",
                    },
                    TrailingDelta {
                        min_trailing_above_delta: 10,
                        max_trailing_above_delta: 2000,
                        min_trailing_below_delta: 10,
                        max_trailing_below_delta: 2000,
                    },
                    PercentPriceBySide {
                        bid_multiplier_up: "1.25",
                        bid_multiplier_down: "0.2",
                        ask_multiplier_up: "5",
                        ask_multiplier_down: "0.75",
                        avg_price_mins: 5,
                    },
                    MaxNumOrders {
                        max_num_orders: 200,
                    },
                    MaxNumAlgoOrders {
                        max_num_algo_orders: 5,
                    },
                ],
                permissions: [
                    "SPOT",
                ],
                default_self_trade_prevention_mode: "EXPIRE_MAKER",
                allowed_self_trade_prevention_modes: [
                    "EXPIRE_TAKER",
                    "EXPIRE_MAKER",
                    "EXPIRE_BOTH",
                ],
            },
        ],
    },
*/
