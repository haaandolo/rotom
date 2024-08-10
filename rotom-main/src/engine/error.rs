use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Failed to build engine due to missing: {0}")]
    BuilderIncomplete(&'static str)
}