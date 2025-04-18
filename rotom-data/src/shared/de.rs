use serde::{Deserialize, Deserializer, Serializer};

// Deserialize a `String` as the desired type.
pub fn de_str<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::de::Deserializer<'de>,
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let data: &str = serde::de::Deserialize::deserialize(deserializer)?;
    data.parse::<T>().map_err(serde::de::Error::custom)
}

#[derive(Deserialize)]
#[serde(untagged)]
enum StrOrFloat<'a> {
    Str(&'a str),
    Float(f64),
}

pub fn de_flexi_float<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = StrOrFloat::deserialize(deserializer)?;
    match value {
        StrOrFloat::Str(s) => s.parse().map_err(serde::de::Error::custom),
        StrOrFloat::Float(f) => Ok(f),
    }
}

// Deserialize a symbol to a be lowercase and non-snake or camel case
pub fn de_str_symbol<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::de::Deserializer<'de>,
    T: std::str::FromStr + std::iter::FromIterator<char>,
    T::Err: std::fmt::Display,
{
    let data: &str = serde::de::Deserialize::deserialize(deserializer)?;
    Ok(data
        .chars()
        .filter(|&c| c != '_')
        .map(|c| c.to_ascii_lowercase())
        .collect())
}

// Deserialize date
pub fn datetime_utc_from_epoch_duration(
    duration: std::time::Duration,
) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::<chrono::Utc>::from(std::time::UNIX_EPOCH + duration)
}

// Deserialize a `u64` milliseconds value as `DateTime<Utc>`.
pub fn de_u64_epoch_ms_as_datetime_utc<'de, D>(
    deserializer: D,
) -> Result<chrono::DateTime<chrono::Utc>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    serde::de::Deserialize::deserialize(deserializer).map(|epoch_ms| {
        datetime_utc_from_epoch_duration(std::time::Duration::from_millis(epoch_ms))
    })
}

// Deserialize a &str "u64" milliseconds value as `DateTime<Utc>`.
pub fn de_str_u64_epoch_ms_as_datetime_utc<'de, D>(
    deserializer: D,
) -> Result<chrono::DateTime<chrono::Utc>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    de_str(deserializer).map(|epoch_ms| {
        datetime_utc_from_epoch_duration(std::time::Duration::from_millis(epoch_ms))
    })
}

// Deserialize a &str "f64" milliseconds value as `DateTime<Utc>`.
pub fn de_str_f64_epoch_ms_as_datetime_utc<'de, D>(
    deserializer: D,
) -> Result<chrono::DateTime<chrono::Utc>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    de_str(deserializer).map(|epoch_ms: f64| {
        datetime_utc_from_epoch_duration(std::time::Duration::from_millis(epoch_ms as u64))
    })
}

// Deserialise string u64 to DateTime<Utc> in nanoseconds
pub fn de_str_u64_epoch_ns_as_datetime_utc<'de, D>(
    deserializer: D,
) -> Result<chrono::DateTime<chrono::Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    de_str(deserializer).map(chrono::DateTime::<chrono::Utc>::from_timestamp_nanos)
}

// Deserialise string u64 to DateTime<Utc> in nanoseconds
pub fn de_u64_epoch_ns_as_datetime_utc<'de, D>(
    deserializer: D,
) -> Result<chrono::DateTime<chrono::Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    serde::de::Deserialize::deserialize(deserializer)
        .map(|epoch_ns| datetime_utc_from_epoch_duration(std::time::Duration::from_nanos(epoch_ns)))
}

// Deserialize a &str "f64" seconds value as `DateTime<Utc>`.
pub fn de_str_f64_epoch_s_as_datetime_utc<'de, D>(
    deserializer: D,
) -> Result<chrono::DateTime<chrono::Utc>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    de_str(deserializer).map(|epoch_s: f64| {
        datetime_utc_from_epoch_duration(std::time::Duration::from_secs_f64(epoch_s))
    })
}

// Deserialise a optional str. For example value to deserialise is "69.69". This
// de will return Some(69.69) if exists. If value to derserialise is "1000", it
// will return Some(1000.0) if exists. The value you are de has to be a string.
// But the optional value you want to de can be anything
pub fn de_str_optional<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let s: Option<&str> = Option::deserialize(deserializer)?;
    match s {
        Some(s) => s.parse::<T>().map(Some).map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

// Serialise value to upper case
pub fn se_uppercase<S, T: ToString>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.to_string().to_uppercase())
}

pub fn de_string_or_i32<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNumber {
        String(String),
        Number(i32),
    }

    match StringOrNumber::deserialize(deserializer)? {
        StringOrNumber::String(s) => s.parse::<i32>().map_err(serde::de::Error::custom),
        StringOrNumber::Number(n) => Ok(n),
    }
}

// Deserialise a value to uppercase
pub fn de_uppercase<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer).map(|s| s.to_uppercase())
}

// Deserialise a value to uppercase
pub fn de_lowercase<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer).map(|s| s.to_lowercase())
}
