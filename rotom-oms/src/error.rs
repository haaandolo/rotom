use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrderManagmentSystemError {
    #[error("Failed to build struct due to missing attribute: {0}")]
    BuilderIncomplete(&'static str),
}
