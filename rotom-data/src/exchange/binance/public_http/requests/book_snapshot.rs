use serde::Deserialize;

use crate::assets::level::Level;
use crate::shared::de::deserialize_non_empty_vec;
use crate::shared::utils::snapshot_symbol_default_value;

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
