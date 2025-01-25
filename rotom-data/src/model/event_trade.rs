use serde::Deserialize;

use crate::assets::level::Level;

use super::SubKind;

/*----- */
// Trades
/*----- */

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Default)]
pub struct Trade;

impl SubKind for Trade {
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
// Trades
/*----- */
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Default)]
pub struct Trades;

impl SubKind for Trades {
    type Event = Vec<EventTrade>;
}
