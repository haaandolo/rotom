use error::EngineError;
use tokio::sync::oneshot;

use crate::{data::Market, oms::position::Position};

pub mod trader;
pub mod error;

/*----- */
// Commands that can be sent trader
/*----- */
#[derive(Debug)]
pub enum Command {
    FetchOpenPositions(oneshot::Sender<Result<Vec<Position>, EngineError>>),
    Terminate(String),
    ExitAllPositions,
    ExitPosition(Market),
}