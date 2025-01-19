use thiserror::Error;

use super::persistence::error::RepositoryError;

#[derive(Debug, Error)]
pub enum PortfolioError {
    #[error("Failed to build struct due to missing attributes: {0}")]
    BuilderIncomplete(&'static str),

    #[error("Failed to parse Position entry Side due to ambiguous fill quantity & Decision.")]
    ParseEntrySide,

    #[error("Cannot exit Position with an entry decision FillEvent.")]
    CannotEnterPositionWithExitFill,

    #[error("Cannot exit Position with an entry decision FillEvent.")]
    CannotExitPositionWithEntryFill,

    #[error("Cannot generate PositionExit from Position that has not been exited")]
    PositionExit,

    #[error("Failed to interact with repository")]
    RepositoryInteraction(#[from] RepositoryError),
}
