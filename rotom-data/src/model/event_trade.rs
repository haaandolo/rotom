use serde::Deserialize;

use crate::assets::level::Level;

use super::SubKind;

/*----- */
// Trades
/*----- */

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Default)]
pub struct Trades;

impl SubKind for Trades {
    type Event = EventTrade;
}

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize)]
pub struct EventTrade {
    pub trade: Level,
    pub is_buy: bool,
}

impl EventTrade {
    pub fn new(trade: Level, is_buy: bool) -> Self {
        Self { trade, is_buy }
    }
}

/*----- */
// Aggregated trades
/*----- */
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Default)]
pub struct AggTrades;

impl SubKind for AggTrades {
    type Event = EventTrade;
}

/*----- */
// Trades Vec
/*----- */
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Default)]
pub struct TradesVec;

impl SubKind for TradesVec {
    type Event = Vec<EventTrade>;
}
