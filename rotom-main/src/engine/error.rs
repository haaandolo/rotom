use thiserror::Error;

use crate::portfolio::repository::error::RepositoryError;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Failed to build engine due to missing: {0}")]
    BuilderIncomplete(&'static str),

    #[error("Failed to interact with repository")]
    RepositoryInteractionError(#[from] RepositoryError),
}