use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExecutionError {
    #[error("Failed to build struct due to missing attribute: {0}")]
    BuilderIncomplete(&'static str),
}

#[derive(Error, Debug)]
pub enum RequestBuildError {
    #[error("{exchange} failed to build for {request} request")]
    BuilderError {
        exchange: &'static str,
        request: &'static str,
    },
}
