use async_trait::async_trait;

use crate::engine::Command;

pub mod arb_trader;
pub mod single_trader;

/*----- */
// Trader trait
/*----- */
#[async_trait]
pub trait TraderRun {
    fn receive_remote_command(&mut self) -> Option<Command>;

    async fn run(self);
}
