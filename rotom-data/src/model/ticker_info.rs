// This is a generalised interface for information regarding any asset.
// Add more fields as we go
#[derive(Debug)]
pub struct TickerInfo {
    // Represents how much the quantity of base asset is allowed to +ve or -ve
    // E.g. 0.01, this means the base asset quanitity has to be fixed at 2 dp
    // This is useful for opening new orders. Some exchanges give precision value
    // as int's i.e. 2 means 0.01 so we need to make the required coversions
    pub price_precision: f64,
    // Represents how much the price of base asset is allowed to +ve or -ve
    // E.g. 0.01, this means the base asset price can tick by intervals of
    // 0.01 (1.41 <- 1.42 <- # 1.43 # -> 1.44 -> 1.45). This is useful when
    // sending market orders and you want to send orders in usdt i.e. the
    // quote asset price and not the base asset quantity. Some exchanges give
    // precision value as int's i.e. 2 means 0.01 so we need to make the
    // required coversions
    pub quantity_precision: f64,
}
