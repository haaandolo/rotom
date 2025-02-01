use crate::shared::subscription_models::ExchangeId;

#[derive(Debug)]
pub struct NetworkSpecs(pub Vec<NetworkSpecData>);

#[derive(Debug)]
pub struct NetworkSpecData {
    pub coin: String,
    pub exchange: ExchangeId,
    pub chains: Vec<ChainSpecs>,
}

#[derive(Debug, Default)]
pub struct ChainSpecs {
    pub chain_name: String,
    // fee is denominated in the corresponding coin e.g the coin field in NetworkSpecs
    pub fees: f64,
    pub can_deposit: bool,
    pub can_withdraw: bool,
}
