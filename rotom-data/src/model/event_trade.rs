use serde::Deserialize;

use crate::assets::level::Level;

use super::{EventKind, SubKind};

/*----- */
// Trade Event
/*----- */
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
// Trade
/*----- */
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Default)]
pub struct Trade;

impl SubKind for Trade {
    const EVENTKIND: EventKind = EventKind::Trade;
    type Event = EventTrade;
}

/*----- */
// Aggregated trades
/*----- */
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Default)]
pub struct AggTrades;

impl SubKind for AggTrades {
    const EVENTKIND: EventKind = EventKind::Trade;
    type Event = EventTrade;
}

/*----- */
// Trades
/*----- */
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Default)]
pub struct Trades;

impl SubKind for Trades {
    const EVENTKIND: EventKind = EventKind::Trade;
    type Event = Vec<EventTrade>;
}
