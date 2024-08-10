use tokio::sync::mpsc::{self, UnboundedReceiver};

use super::{Feed, MarketGenerator};

/*----- */
// Market feed
/*----- */
#[derive(Debug)]
pub struct MarketFeed<Event> {
    pub market_rx: UnboundedReceiver<Event>
}

impl<Event> MarketFeed<Event> {
 pub fn new(market_rx: UnboundedReceiver<Event>) -> Self {
        Self { market_rx }
    }
}

/*----- */
// Impl MarketGenerator
/*----- */
impl<Event> MarketGenerator<Event> for MarketFeed<Event> {
    fn next(&mut self) -> super::Feed<Event> {
        loop {
            match self.market_rx.try_recv() {
                Ok(event) => break Feed::Next(event),
                Err(mpsc::error::TryRecvError::Empty) => continue,
                Err(mpsc::error::TryRecvError::Disconnected) => break Feed::Finished
            }
        }
    }
}