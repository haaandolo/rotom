pub mod single_trader;
pub mod spot_arb_trader;

use async_trait::async_trait;

use crate::engine::Command;

/*----- */
// Trader trait
/*----- */
#[async_trait]
pub trait TraderRun {
    fn receive_remote_command(&mut self) -> Option<Command>;

    async fn run(self);
}
