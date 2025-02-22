use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fmt::Display};

use crate::shared::de::de_flexi_float;

#[derive(Default, Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Level {
    #[serde(deserialize_with = "de_flexi_float")]
    pub price: f64,
    #[serde(deserialize_with = "de_flexi_float")]
    pub size: f64,
}

impl Level {
    pub fn new(price: f64, size: f64) -> Self {
        Self { price, size }
    }

    // Used for testing
    pub fn new_random() -> Self {
        let price_random = rand::thread_rng().gen::<f64>();
        let size_random = rand::thread_rng().gen::<f64>();

        Self {
            price: price_random,
            size: size_random,
        }
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
