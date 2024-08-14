use chrono::{DateTime, Utc};

pub mod live;
pub mod error;

pub trait MarketGenerator<Event> {
    fn next(&mut self) -> Feed<Event>;
} 

#[derive(Debug)]
pub enum Feed<Event> {
    Next(Event),
    UnHealthy,
    Finished
}

/*----- */
// Market metadata
/*----- */
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct MarketMeta {
    pub close: f64,
    pub time: DateTime<Utc>
}

impl Default for MarketMeta {
    fn default() -> Self {
        Self {
            close: 100.0,
            time: Utc::now()
        }
    }
}