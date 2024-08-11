use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::{
    assets::level::Level, error::SocketError, event_models::{event_trade::EventTrade, market_event::MarketEvent}, exchange::Identifier, shared::{
        de::{de_str, de_u64_epoch_ms_as_datetime_utc, deserialize_non_empty_vec},
        subscription_models::ExchangeId,
        utils::snapshot_symbol_default_value,
    }, streams::validator::Validator
};

/*----- */
// Snapshot
/*----- */
#[derive(PartialEq, PartialOrd, Debug, Deserialize)]
pub struct BinanceSpotSnapshot {
    #[serde(default = "snapshot_symbol_default_value")]
    symbol: String,
    #[serde(rename = "lastUpdateId")]
    pub last_update_id: u64,
    #[serde(deserialize_with = "deserialize_non_empty_vec")]
    pub bids: Option<Vec<Level>>,
    #[serde(deserialize_with = "deserialize_non_empty_vec")]
    pub asks: Option<Vec<Level>>,
}

/*----- */
// Orderbook L2
/*----- */
#[derive(PartialEq, PartialOrd, Debug, Deserialize, Default)]
pub struct BinanceSpotBookUpdate {
    #[serde(alias = "s")]
    #[serde(alias = "p")]
    pub symbol: String,
    #[serde(alias = "U")]
    pub first_update_id: u64,
    #[serde(alias = "u")]
    pub last_update_id: u64,
    #[serde(alias = "b")]
    #[serde(deserialize_with = "deserialize_non_empty_vec")]
    pub bids: Option<Vec<Level>>,
    #[serde(alias = "a")]
    #[serde(deserialize_with = "deserialize_non_empty_vec")]
    pub asks: Option<Vec<Level>>,
}

impl Identifier<String> for BinanceSpotBookUpdate {
    fn id(&self) -> String {
        self.symbol.clone()
    }
}

/*----- */
// Trade
/*----- */
#[derive(PartialEq, PartialOrd, Debug, Deserialize, Default)]
pub struct BinanceTrade {
    pub symbol: String,
    #[serde(alias = "T", deserialize_with = "de_u64_epoch_ms_as_datetime_utc")]
    pub timestamp: DateTime<Utc>,
    #[serde(alias = "t")]
    pub id: u64,
    #[serde(alias = "p", deserialize_with = "de_str")]
    pub price: f64,
    #[serde(alias = "q", deserialize_with = "de_str")]
    pub amount: f64,
    #[serde(alias = "m")]
    pub side: bool,
}

impl From<BinanceTrade> for MarketEvent<EventTrade> {
    fn from(event: BinanceTrade) -> Self {
        Self {
            exchange_time: event.timestamp,
            received_time: Utc::now(),
            exchange: ExchangeId::BinanceSpot,
            symbol: event.symbol,
            event_data: EventTrade::new(Level::new(event.price, event.amount), event.side),
        }
    }
}

/*----- */
// Subscription response
/*----- */
#[derive(Debug, Deserialize, PartialEq)]
pub struct BinanceSubscriptionResponse {
    pub result: Option<String>,
    pub id: u32,
}

impl Validator for BinanceSubscriptionResponse {
    fn validate(self) -> Result<Self, SocketError>
    where
        Self: Sized,
    {
        if self.result.is_none() {
            Ok(self)
        } else {
            Err(SocketError::Subscribe(
                "received failure subscription response".to_owned(),
            ))
        }
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged, rename_all = "snake_case")]
pub enum BinanceMessage {
    Book(BinanceSpotBookUpdate),
    Snapshot(BinanceSpotSnapshot),
    Trade(BinanceTrade),
}

/*----- */
// Ticker info
/*----- */
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
        tick_size: f64,
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
        #[serde(rename = "minQty")]
        min_qty: String,
        #[serde(rename = "maxQty")]
        max_qty: String,
        #[serde(rename = "stepSize")]
        step_size: String,
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

// /*----- */
// // Tests
// /*----- */
// #[cfg(test)]
// mod test {
//     use super::*;

//     #[test]
//     fn snapshot_de() {
//         let snapshot =  "{\"lastUpdateId\":3476852730,\"bids\":[[\"0.00914500\",\"2.18100000\"],[\"0.00914400\",\"8.12300000\"]],\"asks\":[[\"0.00914600\",\"312.94200000\"],[\"0.00914700\",\"7.30100000\"]]}";
//         let snapshot_de = serde_json::from_str::<BinanceSnapshot>(snapshot).unwrap();
//         let snapshot_expected = BinanceSnapshot {
//             symbol: "snapshot".to_string(),
//             timestamp: current_timestamp_utc(),
//             last_update_id: 3476852730,
//             bids: Some(vec![
//                 Level::new(0.00914500, 2.18100000),
//                 Level::new(0.00914400, 8.12300000),
//             ]),
//             asks: Some(vec![
//                 Level::new(0.00914600, 312.94200000),
//                 Level::new(0.00914700, 7.30100000),
//             ]),
//         };
//         assert_eq!(snapshot_de, snapshot_expected)
//     }

//     #[test]
//     fn snapshot_to_event() {
//         let current_timestamp = current_timestamp_utc();
//         let snapshot_struct = BinanceSnapshot {
//             symbol: "snapshot".to_string(),
//             timestamp: current_timestamp,
//             last_update_id: 3476852730,
//             bids: Some(vec![
//                 Level::new(0.00914500, 2.18100000),
//                 Level::new(0.00914400, 8.12300000),
//             ]),
//             asks: Some(vec![
//                 Level::new(0.00914600, 312.94200000),
//                 Level::new(0.00914700, 7.30100000),
//             ]),
//         };

//         let event_expected = Event::new(
//             "snapshot".to_string(),
//             current_timestamp,
//             3476852730,
//             Some(vec![
//                 Level::new(0.00914500, 2.18100000),
//                 Level::new(0.00914400, 8.12300000),
//             ]),
//             Some(vec![
//                 Level::new(0.00914600, 312.94200000),
//                 Level::new(0.00914700, 7.30100000),
//             ]),
//             None,
//             None,
//         );

//         let event_from = Event::from(snapshot_struct);

//         assert_eq!(event_from, event_expected);
//     }

//     #[test]
//     fn orderbook_de() {
//         let orderbook = "{\"e\":\"depthupdate\",\"E\":1718097006844,\"s\":\"btcusdt\",\"U\":47781538300,\"u\":47781538304,\"b\":[[\"67543.58000000\",\"0.03729000\"],[\"67527.08000000\",\"8.71242000\"]],\"a\":[]}";
//         let orderbook_de = serde_json::from_str::<BinanceBook>(orderbook).unwrap();
//         let orderbook_expected = BinanceBook {
//             symbol: "btcusdt".to_string(),
//             timestamp: 1718097006844,
//             first_update_id: 47781538300,
//             bids: Some(vec![
//                 Level::new(67543.58000000, 0.03729000),
//                 Level::new(67527.08000000, 8.71242000),
//             ]),
//             asks: None,
//         };

//         println!("{:#?}", orderbook_expected);

//         assert_eq!(orderbook_de, orderbook_expected)
//     }

//     #[test]
//     fn orderbook_to_event() {
//         let orderbook_struct = BinanceBook {
//             symbol: "btcusdt".to_string(),
//             timestamp: 1718097006844,
//             first_update_id: 47781538300,
//             bids: Some(vec![
//                 Level::new(67543.58000000, 0.03729000),
//                 Level::new(67527.08000000, 8.71242000),
//             ]),
//             asks: None,
//         };

//         let event_expected = Event::new(
//             "btcusdt".to_string(),
//             1718097006844,
//             47781538300,
//             Some(vec![
//                 Level::new(67543.58000000, 0.03729000),
//                 Level::new(67527.08000000, 8.71242000),
//             ]),
//             None,
//             None,
//             None,
//         );

//         let event_from = Event::from(orderbook_struct);

//         assert_eq!(event_from, event_expected)
//     }

//     #[test]
//     fn trade_de() {
//         let trade = "{\"e\":\"trade\",\"E\":1718097131139,\"s\":\"BTCUSDT\",\"t\":3631373609,\"p\":\"67547.10000000\",\"q\":\"0.00100000\",\"b\":27777962514,\"a\":27777962896,\"T\":1718097131138,\"m\":true,\"M\":true}";
//         let trade_de = serde_json::from_str::<BinanceTrade>(trade).unwrap();
//         let trade_expected = BinanceTrade {
//             symbol: "btcusdt".to_string(),
//             timestamp: 1718097131138,
//             id: 3631373609,
//             price: 67547.10000000,
//             amount: 0.00100000,
//             side: true,
//         };
//         assert_eq!(trade_de, trade_expected)
//     }

//     #[test]
//     fn trade_to_event() {
//         let trade_struct = BinanceTrade {
//             symbol: "btcusdt".to_string(),
//             timestamp: 1718097131138,
//             id: 3631373609,
//             price: 67547.10000000,
//             amount: 0.00100000,
//             side: true,
//         };

//         let event_expected = Event::new(
//             "btcusdt".to_string(),
//             1718097131138,
//             3631373609,
//             None,
//             None,
//             Some(Level::new(67547.10000000, 0.00100000)),
//             Some(true),
//         );

//         let event_from = Event::from(trade_struct);

//         assert_eq!(event_from, event_expected)
//     }

//     #[test]
//     fn expected_response_de() {
//         let expected_response = "{\"result\":null,\"id\":1}";
//         let expected_response_de =
//             serde_json::from_str::<BinanceSubscriptionResponse>(expected_response).unwrap();
//         let expected_response_struct = BinanceSubscriptionResponse {
//             result: None,
//             id: 1,
//         };
//         assert_eq!(expected_response_de, expected_response_struct)
//     }
// }
