use chrono::Utc;
use serde::Deserialize;

use crate::data::shared::{
    de::de_str,
    orderbook::{Event, Level},
};

/*----- */
// Models
/*----- */
#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize)]
pub struct BinanceSnapshot {
    #[serde(default = "current_timestamp_utc")]
    pub timestamp: u64,
    #[serde(rename = "lastUpdateId")]
    pub last_update_id: u64,
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
}

fn current_timestamp_utc() -> u64 {
    Utc::now().timestamp() as u64
}

impl From<BinanceSnapshot> for Event {
    fn from(value: BinanceSnapshot) -> Self {
        Event::new(
            value.timestamp,
            value.last_update_id,
            Some(value.bids),
            Some(value.asks),
            None,
            None,
        )
    }
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize)]
pub struct BinanceBookUpdate {
    #[serde(alias = "E")]
    pub timestamp: u64,
    #[serde(alias = "U")]
    pub first_update_id: u64,
    #[serde(alias = "b")]
    pub bids: Vec<Level>,
    #[serde(alias = "a")]
    pub asks: Vec<Level>,
}

impl From<BinanceBookUpdate> for Event {
    fn from(value: BinanceBookUpdate) -> Self {
        Event::new(
            value.timestamp,
            value.first_update_id,
            Some(value.bids),
            Some(value.asks),
            None,
            None,
        )
    }
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize)]
pub struct BinanceTradeUpdate {
    #[serde(alias = "T")]
    pub timestamp: u64,
    #[serde(alias = "t")]
    pub id: u64,
    #[serde(alias = "p", deserialize_with = "de_str")]
    pub price: f64,
    #[serde(alias = "q", deserialize_with = "de_str")]
    pub amount: f64,
    #[serde(alias = "m")]
    pub side: bool,
}

impl From<BinanceTradeUpdate> for Event {
    fn from(value: BinanceTradeUpdate) -> Self {
        Event::new(
            value.timestamp,
            value.id,
            None,
            None,
            Some(Level::new(value.price, value.amount)),
            Some(value.side),
        )
    }
}

/*----- */
// Tests
/*----- */
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn snapshot_de() {
        let snapshot =  "{\"lastUpdateId\":3476852730,\"bids\":[[\"0.00914500\",\"2.18100000\"],[\"0.00914400\",\"8.12300000\"]],\"asks\":[[\"0.00914600\",\"312.94200000\"],[\"0.00914700\",\"7.30100000\"]]}";
        let snapshot_de = serde_json::from_str::<BinanceSnapshot>(snapshot).unwrap();
        let snapshot_expected = BinanceSnapshot {
            timestamp: current_timestamp_utc(),
            last_update_id: 3476852730,
            bids: vec![
                Level::new(0.00914500, 2.18100000),
                Level::new(0.00914400, 8.12300000),
            ],
            asks: vec![
                Level::new(0.00914600, 312.94200000),
                Level::new(0.00914700, 7.30100000),
            ],
        };
        assert_eq!(snapshot_de, snapshot_expected)
    }

    #[test]
    fn snapshot_to_event() {
        let current_timestamp = current_timestamp_utc();
        let snapshot_struct = BinanceSnapshot {
            timestamp: current_timestamp,
            last_update_id: 3476852730,
            bids: vec![
                Level::new(0.00914500, 2.18100000),
                Level::new(0.00914400, 8.12300000),
            ],
            asks: vec![
                Level::new(0.00914600, 312.94200000),
                Level::new(0.00914700, 7.30100000),
            ],
        };

        let event_expected = Event::new(
            current_timestamp,
            3476852730,
            Some(vec![
                Level::new(0.00914500, 2.18100000),
                Level::new(0.00914400, 8.12300000),
            ]),
            Some(vec![
                Level::new(0.00914600, 312.94200000),
                Level::new(0.00914700, 7.30100000),
            ]),
            None,
            None,
        );

        let event_from = Event::from(snapshot_struct);

        assert_eq!(event_from, event_expected);
    }

    #[test]
    fn orderbook_de() {
        let orderbook = "{\"e\":\"depthupdate\",\"E\":1718097006844,\"s\":\"btcusdt\",\"U\":47781538300,\"u\":47781538304,\"b\":[[\"67543.58000000\",\"0.03729000\"],[\"67527.08000000\",\"8.71242000\"]],\"a\":[]}";
        let orderbook_de = serde_json::from_str::<BinanceBookUpdate>(orderbook).unwrap();
        let orderbook_expected = BinanceBookUpdate {
            timestamp: 1718097006844,
            first_update_id: 47781538300,
            bids: vec![
                Level::new(67543.58000000, 0.03729000),
                Level::new(67527.08000000, 8.71242000),
            ],
            asks: vec![],
        };

        assert_eq!(orderbook_de, orderbook_expected)
    }

    #[test]
    fn orderbook_to_event() {
        let orderbook_struct = BinanceBookUpdate {
            timestamp: 1718097006844,
            first_update_id: 47781538300,
            bids: vec![
                Level::new(67543.58000000, 0.03729000),
                Level::new(67527.08000000, 8.71242000),
            ],
            asks: vec![],
        };

        let event_expected = Event::new(
            1718097006844,
            47781538300,
            Some(vec![
                Level::new(67543.58000000, 0.03729000),
                Level::new(67527.08000000, 8.71242000),
            ]),
            Some(vec![]),
            None,
            None,
        );

        let event_from = Event::from(orderbook_struct);

        assert_eq!(event_from, event_expected)
    }

    #[test]
    fn trade_de() {
        let trade = "{\"e\":\"trade\",\"E\":1718097131139,\"s\":\"BTCUSDT\",\"t\":3631373609,\"p\":\"67547.10000000\",\"q\":\"0.00100000\",\"b\":27777962514,\"a\":27777962896,\"T\":1718097131138,\"m\":true,\"M\":true}";
        let trade_de = serde_json::from_str::<BinanceTradeUpdate>(trade).unwrap();
        let trade_expected = BinanceTradeUpdate {
            timestamp: 1718097131138,
            id: 3631373609,
            price: 67547.10000000,
            amount: 0.00100000,
            side: true,
        };
        assert_eq!(trade_de, trade_expected)
    }

    #[test]
    fn trade_to_event() {
        let trade_struct = BinanceTradeUpdate {
            timestamp: 1718097131138,
            id: 3631373609,
            price: 67547.10000000,
            amount: 0.00100000,
            side: true,
        };

        let event_expected = Event::new(
            1718097131138,
            3631373609,
            None,
            None,
            Some(Level::new(67547.10000000, 0.00100000)),
            Some(true),
        );

        let event_from = Event::from(trade_struct);

        assert_eq!(event_from, event_expected)
    }
}