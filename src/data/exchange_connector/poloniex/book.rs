use serde::Deserialize;

use crate::data::shared::{de::de_str, orderbook::{Event, Level}};


/*----- */
// Models
/*----- */
#[derive(Deserialize, Debug, PartialEq)]
pub struct PoloniexBookData {
    pub symbol: String,
    #[serde(alias = "createTime")]
    pub timestamp: u64,
    pub asks: Vec<Level>,
    pub bids: Vec<Level>,
    #[serde(alias = "lastId")]
    pub last_id: u64,
    pub id: u64,
    pub ts: u64,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct PoloniexBookUpdate {
    pub data: Vec<PoloniexBookData>,
}

impl From<PoloniexBookUpdate> for Event {
    fn from(mut value: PoloniexBookUpdate) -> Self {
        let data = value.data.remove(0);
        Event::new(
            data.timestamp,
            data.id,
            Some(data.bids),
            Some(data.asks),
            None,
            None
        )    
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct PoloniexTradeData {
    pub symbol: String,
    #[serde(deserialize_with = "de_str")]
    pub amount: f64,
    #[serde(deserialize_with = "de_str")]
    pub quantity: f64,
    #[serde(alias = "takerSide", deserialize_with = "de_buyer_is_maker_poloniex")]
    pub is_buy: bool,
    #[serde(alias = "createTime")]
    pub timestamp: u64,
    #[serde(deserialize_with = "de_str")]
    pub price: f64,
    #[serde(deserialize_with = "de_str")]
    pub id: u64,
    pub ts: u64,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct PoloneixTradeUpdate {
    pub data: Vec<PoloniexTradeData>,
}

impl From<PoloneixTradeUpdate> for Event {
    fn from(mut value: PoloneixTradeUpdate) -> Self {
        let data = value.data.remove(0);
        Event::new(
            data.timestamp,
            data.id,
            None,
            None,
            Some(Level::new(data.price, data.quantity)),
            Some(data.is_buy)
        )
    }
}

/*----- */
// Exchange specific de
/*----- */
#[allow(clippy::needless_bool)]
pub fn de_buyer_is_maker_poloniex<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    <&str as Deserialize>::deserialize(deserializer).map(|buyer_is_maker| {
        if buyer_is_maker == "sell" {
            false
        } else {
            true
        }
    })
}

/*----- */
// Tests
/*----- */
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn orderbook_de() {
        let orderbook = "{\"channel\":\"book_lv2\",\"data\":[{\"symbol\":\"BTC_USDT\",\"createTime\":1718096579424,\"asks\":[],\"bids\":[[\"67546.83\",\"0.027962\"],[\"67301.78\",\"0\"]],\"lastId\":1051076040,\"id\":1051076041,\"ts\":1718096579435}],\"action\":\"update\"}";
        let orderbook_de = serde_json::from_str::<PoloniexBookUpdate>(orderbook).unwrap();
        let orderbook_data_expected = PoloniexBookData {
            symbol: "BTC_USDT".to_string(),
            timestamp: 1718096579424,
            asks: vec![],
            bids: vec![Level::new(67546.83, 0.027962), Level::new(67301.78, 0.0)],
            last_id: 1051076040,
            id: 1051076041,
            ts: 1718096579435,
        };

        let orderbook_expected = PoloniexBookUpdate {
            data: vec![orderbook_data_expected],
        };

        assert_eq!(orderbook_de, orderbook_expected)
    }

    #[test]
    fn orderbook_to_event() {
        let orderbook_data = PoloniexBookData {
            symbol: "BTC_USDT".to_string(),
            timestamp: 1718096579424,
            asks: vec![],
            bids: vec![Level::new(67546.83, 0.027962), Level::new(67301.78, 0.0)],
            last_id: 1051076040,
            id: 1051076041,
            ts: 1718096579435,
        };

        let orderbook_struct = PoloniexBookUpdate {
            data: vec![orderbook_data],
        };

        let event_expected = Event::new(
            1718096579424,
            1051076041,
            Some(vec![Level::new(67546.83, 0.027962), Level::new(67301.78, 0.0)]),
            Some(vec![]),
            None,
            None
        );

        let event_from = Event::from(orderbook_struct);

        assert_eq!(event_from, event_expected)
    }

    #[test]
    fn trade_de() {
        let trade = "{\"channel\":\"trades\",\"data\":[{\"symbol\":\"BTC_USDT\",\"amount\":\"1684.53544514\",\"quantity\":\"0.024914\",\"takerSide\":\"sell\",\"createTime\":1718096866390,\"price\":\"67614.01\",\"id\":\"95714554\",\"ts\":1718096866402}]}";
        let trade_de = serde_json::from_str::<PoloneixTradeUpdate>(trade).unwrap();
        let trade_data_expected = PoloniexTradeData {
            symbol: "BTC_USDT".to_string(),
            amount: 1684.53544514,
            quantity: 0.024914,
            is_buy: false,
            timestamp: 1718096866390,
            price: 67614.01,
            id: 95714554,
            ts: 1718096866402,
        };

        let trade_expected = PoloneixTradeUpdate {
            data: vec![trade_data_expected]
        };

        assert_eq!(trade_de, trade_expected)
    }

    #[test]
    fn trade_to_event() {
        let trade_data = PoloniexTradeData {
            symbol: "BTC_USDT".to_string(),
            amount: 1684.53544514,
            quantity: 0.024914,
            is_buy: false,
            timestamp: 1718096866390,
            price: 67614.01,
            id: 95714554,
            ts: 1718096866402,
        };

        let trade_struct = PoloneixTradeUpdate {
            data: vec![trade_data]
        };

        let event_expected = Event::new(
            1718096866390,
            95714554,
            None,
            None,
            Some(Level::new(67614.01, 0.024914)),
            Some(false)
        );

        let event_from = Event::from(trade_struct);

        assert_eq!(event_from, event_expected)
    }

    #[test]
    fn snapshot_de() {
        let snapshot = "{\"channel\":\"book_lv2\",\"data\":[{\"symbol\":\"BTC_USDT\",\"createTime\":1718096578118,\"asks\":[[\"67547.43\",\"0.039788\"]],\"bids\":[[\"67546.15\",\"0.238432\"]],\"lastId\":1051076022,\"id\":1051076023,\"ts\":1718096578303}],\"action\":\"snapshot\"}";
        let snapshot_de = serde_json::from_str::<PoloniexBookUpdate>(snapshot).unwrap();
        let snapshot_data_expected = PoloniexBookData {
            symbol: "BTC_USDT".to_string(),
            timestamp: 1718096578118,
            asks: vec![Level::new(67547.43, 0.039788)],
            bids: vec![Level::new(67546.15, 0.238432)],
            last_id: 1051076022,
            id: 1051076023,
            ts: 1718096578303,
        };

        let snapshot_expected = PoloniexBookUpdate {
            data: vec![snapshot_data_expected],
        };

        assert_eq!(snapshot_de, snapshot_expected)
    }

    #[test]
    fn snapshot_to_event() {
        let snapshot_data = PoloniexBookData {
            symbol: "BTC_USDT".to_string(),
            timestamp: 1718096578118,
            asks: vec![Level::new(67547.43, 0.039788)],
            bids: vec![Level::new(67546.15, 0.238432)],
            last_id: 1051076022,
            id: 1051076023,
            ts: 1718096578303,
        };

        let snapshot_struct = PoloniexBookUpdate {
            data: vec![snapshot_data],
        };

        let event_expected = Event::new(
            1718096578118,
            1051076023,
            Some(vec![Level::new(67546.15, 0.238432)]),
            Some(vec![Level::new(67547.43, 0.039788)]),
            None,
            None
        );

        let event_from = Event::from(snapshot_struct);

        assert_eq!(event_from, event_expected)
    }
}

/*---------- */
// Examples
/*---------- */
// let polo_book = "{\"channel\":\"book_lv2\",\"data\":[{\"symbol\":\"BTC_USDT\",\"createTime\":1718096579424,\"asks\":[],\"bids\":[[\"67546.83\",\"0.027962\"],[\"67301.78\",\"0\"]],\"lastId\":1051076040,\"id\":1051076041,\"ts\":1718096579435}],\"action\":\"update\"}";
// let polo_snap = "{\"channel\":\"book_lv2\",\"data\":[{\"symbol\":\"BTC_USDT\",\"createTime\":1718096578118,\"asks\":[[\"67547.43\",\"0.039788\"]],\"bids\":[[\"67546.15\",\"0.238432\"]],\"lastId\":1051076022,\"id\":1051076023,\"ts\":1718096578303}],\"action\":\"snapshot\"}";
// let polo_trade = "{\"channel\":\"trades\",\"data\":[{\"symbol\":\"BTC_USDT\",\"amount\":\"1684.53544514\",\"quantity\":\"0.024914\",\"takerSide\":\"sell\",\"createTime\":1718096866390,\"price\":\"67614.01\",\"id\":\"95714554\",\"ts\":1718096866402}]}";
