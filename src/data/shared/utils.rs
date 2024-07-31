use chrono::Utc;

pub fn current_timestamp_utc() -> u64 {
    Utc::now().timestamp_millis() as u64
}

pub fn snapshot_symbol_default_value() -> String {
    String::from("snapshot") 
}

pub fn decimal_places_to_number(places: u8) -> f64 {
    10f64.powi(-(places as i32))
}
