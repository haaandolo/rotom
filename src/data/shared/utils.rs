use chrono::Utc;

pub fn current_timestamp_utc() -> u64 {
    Utc::now().timestamp_millis() as u64
}

pub fn snapshot_symbol_default_value() -> String {
    String::from("snapshot") 
}