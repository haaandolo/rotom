use serde::Deserialize;
use std::{cmp::Ordering, fmt::Display};

use crate::shared::de::de_str;

#[derive(Default, Debug, Clone, Copy, Deserialize)]
pub struct Level {
    #[serde(deserialize_with = "de_str")]
    pub price: f64,
    #[serde(deserialize_with = "de_str")]
    pub size: f64,
}

impl Level {
    pub fn new(price: f64, size: f64) -> Self {
        Self { price, size }
    }
}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for Level {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.price.partial_cmp(&other.price) {
            Some(Ordering::Equal) => self.size.partial_cmp(&other.size),
            other_order => other_order,
        }
    }
}

impl Ord for Level {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialEq for Level {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price && self.size == other.size
    }
}

impl Eq for Level {}

impl Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} : {})", self.price, self.size)
    }
}
