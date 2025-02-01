#[derive(Debug)]
pub struct NetworkSpecs {
    pub coin: String,
    pub chains: Vec<ChainSpecs>,
}

#[derive(Debug)]
pub struct ChainSpecs {
    pub chain_name: String,
    // fee is denominated in the corresponding coin e.g the coin field in NetworkSpecs
    pub fees: f64,
    pub can_deposit: bool,
    pub can_withdraw: bool,
}
