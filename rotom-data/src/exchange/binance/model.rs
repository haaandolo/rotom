use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::{
    assets::level::Level,
    error::SocketError,
    exchange::Identifier,
    model::{
        event_trade::EventTrade,
        market_event::MarketEvent,
        network_info::{ChainSpecs, NetworkSpecData, NetworkSpecs},
        ticker_info::{TickerInfo, TickerSpecs},
    },
    shared::{
        de::{de_str, de_u64_epoch_ms_as_datetime_utc},
        subscription_models::{ExchangeId, Instrument},
        utils::snapshot_symbol_default_value,
    },
    streams::validator::Validator,
};

/*----- */
// Orderbook L2
/*----- */
#[derive(PartialEq, PartialOrd, Debug, Deserialize, Default)]
pub struct BinanceSpotBookUpdate {
    #[serde(alias = "s")]
    pub symbol: String,
    #[serde(alias = "U")]
    pub first_update_id: u64,
    #[serde(alias = "u")]
    pub last_update_id: u64,
    #[serde(alias = "b")]
    pub bids: Vec<Level>,
    #[serde(alias = "a")]
    pub asks: Vec<Level>,
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
    #[serde(alias = "s")]
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

impl Identifier<String> for BinanceTrade {
    fn id(&self) -> String {
        self.symbol.clone()
    }
}

impl From<(BinanceTrade, Instrument)> for MarketEvent<EventTrade> {
    fn from((event, instrument): (BinanceTrade, Instrument)) -> Self {
        Self {
            exchange_time: event.timestamp,
            received_time: Utc::now(),
            exchange: ExchangeId::BinanceSpot,
            instrument,
            event_data: EventTrade::new(Level::new(event.price, event.amount), event.side),
        }
    }
}

/*----- */
// Aggregated Trades
/*----- */
#[derive(PartialEq, PartialOrd, Debug, Deserialize, Default)]
pub struct BinanceAggTrade {
    #[serde(alias = "s")]
    pub symbol: String,
    #[serde(alias = "T", deserialize_with = "de_u64_epoch_ms_as_datetime_utc")]
    pub timestamp: DateTime<Utc>,
    #[serde(alias = "p", deserialize_with = "de_str")]
    pub price: f64,
    #[serde(alias = "q", deserialize_with = "de_str")]
    pub amount: f64,
    #[serde(alias = "m")]
    pub side: bool,
}

impl Identifier<String> for BinanceAggTrade {
    fn id(&self) -> String {
        self.symbol.clone()
    }
}

impl From<(BinanceAggTrade, Instrument)> for MarketEvent<EventTrade> {
    fn from((event, instrument): (BinanceAggTrade, Instrument)) -> Self {
        Self {
            exchange_time: event.timestamp,
            received_time: Utc::now(),
            exchange: ExchangeId::BinanceSpot,
            instrument,
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
                "received failure subscription response BinanceSpot".to_owned(),
            ))
        }
    }
}

/*----- */
// Snapshot
/*----- */
#[derive(PartialEq, PartialOrd, Debug, Deserialize)]
pub struct BinanceSpotSnapshot {
    #[serde(default = "snapshot_symbol_default_value")]
    symbol: String,
    #[serde(rename = "lastUpdateId")]
    pub last_update_id: u64,
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
}

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
    fn from(mut info: BinanceSpotTickerInfo) -> Self {
        let symbol_info = info.symbols.remove(0);
        let (price_precision, min_price) = symbol_info.filters.iter().find_map(|filter| {
            if let Filter::PriceFilter { tick_size, min_price,.. } = filter {
                Some((tick_size.to_owned(), min_price.to_owned()))
            } else {
                None
            }
        }).unwrap_or_else(|| panic!("Price precision for Binance should never panic. This failed for this ticker: {:#?}", info));

        let (quantity_precision, min_quantity) = symbol_info.filters.iter().find_map(|filter| {
            if let Filter::LotSize { step_size, min_qty, .. } = filter {
                Some((step_size.to_owned(), min_qty.to_owned()))
            } else {
                None
            }
        }).unwrap_or_else(|| panic!("Quantity precision for Binance should never panic. This failed for this ticker: {:#?}", info));

        Self {
            symbol: symbol_info.symbol,
            specs: TickerSpecs {
                quantity_precision,
                min_quantity,
                price_precision,
                min_price,
                notional_precision: price_precision,
                min_notional: 5.0, // todo: this is only the case for usdt
            },
        }
    }
}

/*----- */
// Network Info
/*----- */
#[derive(Debug, Deserialize)]
pub struct BinanceNetworkInfo {
    pub coin: String,
    #[serde(rename = "depositAllEnable")]
    pub deposit_all_enable: bool,
    pub free: String,
    pub freeze: String,
    pub ipoable: String,
    pub ipoing: String,
    #[serde(rename = "isLegalMoney")]
    pub is_legal_money: bool,
    pub locked: String,
    pub name: String,
    #[serde(rename = "networkList")]
    pub network_list: Vec<BinanceNetworkList>,
    pub storage: String,
    pub trading: bool,
    #[serde(rename = "withdrawAllEnable")]
    pub withdraw_all_enable: bool,
    pub withdrawing: String,
}

#[derive(Debug, Deserialize)]
pub struct BinanceNetworkList {
    #[serde(rename = "addressRegex")]
    pub address_regex: String,
    pub coin: String,
    #[serde(rename = "depositDesc")]
    pub deposit_desc: String,
    #[serde(rename = "depositEnable")]
    pub deposit_enable: bool,
    #[serde(rename = "isDefault")]
    pub is_default: bool,
    #[serde(rename = "memoRegex")]
    pub memo_regex: String,
    #[serde(rename = "minConfirm")]
    pub min_confirm: i64,
    pub name: String,
    pub network: String,
    #[serde(rename = "specialTips")]
    pub special_tips: String,
    #[serde(rename = "unLockConfirm")]
    pub unlock_confirm: i64,
    #[serde(rename = "withdrawDesc")]
    pub withdraw_desc: String,
    #[serde(rename = "withdrawEnable")]
    pub withdraw_enable: bool,
    #[serde(rename = "withdrawFee", deserialize_with = "de_str")]
    pub withdraw_fee: f64,
    #[serde(rename = "withdrawIntegerMultiple")]
    pub withdraw_integer_multiple: String,
    #[serde(rename = "withdrawMax")]
    pub withdraw_max: String,
    #[serde(rename = "withdrawMin")]
    pub withdraw_min: String,
    #[serde(rename = "withdrawInternalMin")]
    pub withdraw_internal_min: String,
    #[serde(rename = "sameAddress")]
    pub same_address: bool,
    #[serde(rename = "estimatedArrivalTime")]
    pub estimated_arrival_time: i64,
    pub busy: bool,
    #[serde(rename = "contractAddressUrl")]
    pub contract_address_url: Option<String>,
    #[serde(rename = "contractAddress")]
    pub contract_address: Option<String>,
}

impl From<Vec<BinanceNetworkInfo>> for NetworkSpecs {
    fn from(value: Vec<BinanceNetworkInfo>) -> Self {
        let network_spec_data = value
            .iter()
            .map(|coin| {
                let chain_specs = coin
                    .network_list
                    .iter()
                    .map(|chain| ChainSpecs {
                        chain_name: chain.network.clone(),
                        fees: chain.withdraw_fee,
                        can_deposit: chain.deposit_enable,
                        can_withdraw: chain.withdraw_enable,
                    })
                    .collect();

                NetworkSpecData {
                    coin: coin.coin.clone(),
                    exchange: ExchangeId::BinanceSpot,
                    chains: chain_specs,
                }
            })
            .collect::<Vec<_>>();

        NetworkSpecs(network_spec_data)
    }
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
