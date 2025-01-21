#[derive(Debug)]
pub struct HtxChannel(pub &'static str);

impl HtxChannel {
    // pub const TRADES: Self = Self("@trade");
    pub const ORDERBOOKSNAPSHOT: Self = Self(".mbp.refresh.5");
}

impl AsRef<str> for HtxChannel {
    fn as_ref(&self) -> &str {
        self.0
    }
}
