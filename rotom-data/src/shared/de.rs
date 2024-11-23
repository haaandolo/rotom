use serde::{
    de::{SeqAccess, Visitor},
    Deserialize, Deserializer, Serializer,
};
use std::{fmt, marker::PhantomData};

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

// Deserialize a non empty vec else return None
pub fn deserialize_non_empty_vec<'de, D, T>(deserializer: D) -> Result<Option<Vec<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct NonEmptyVecVisitor<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for NonEmptyVecVisitor<T>
    where
        T: Deserialize<'de>,
    {
        type Value = Option<Vec<T>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a sequence")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            match seq.next_element()? {
                Some(first) => {
                    let mut vec = Vec::with_capacity(seq.size_hint().unwrap_or(4));
                    vec.push(first);
                    while let Some(element) = seq.next_element()? {
                        vec.push(element);
                    }
                    Ok(Some(vec))
                }
                None => Ok(None),
            }
        }
    }

    deserializer.deserialize_seq(NonEmptyVecVisitor(PhantomData))
}

// Deserialise a optional str. For example value to deserialise is "69.69". This
// de will return Some(69.69) if exists. If value to derserialise is "1000", it
// will return Some(1000) if exists. None the value you are de has to be a string.
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
