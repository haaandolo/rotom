use std::collections::HashMap;

use serde::Serialize;

use crate::shared::subscription_models::{Coin, ExchangeId};

#[derive(Debug)]
pub struct NetworkSpecs(pub HashMap<(ExchangeId, Coin), NetworkSpecData>);

#[derive(Debug, Clone, Serialize)]
pub struct NetworkSpecData(pub Vec<ChainSpecs>);

#[derive(Debug, Default, Clone, Serialize)]
pub struct ChainSpecs {
    pub chain_name: String,
    // Some exchanges have fixed or percentage fees, if this value is true, then fee is fixed
    pub fee_is_fixed: bool,
    // fee is denominated in the corresponding coin e.g the coin field in NetworkSpecs if it is fixed. Else it is a percentage
    pub fees: f64,
    pub can_deposit: bool,
    pub can_withdraw: bool,
}
