use thiserror::Error;

#[derive(Error, Debug)]
pub enum RequestBuildError {
    #[error("{exchange} failed to build for {request} request")]
    BuilderError {
        exchange: &'static str,
        request: &'static str,
    },

    #[error("{exchange} failed to build for {request} request as {field} is mandatory")]
    MandatoryField {
        exchange: &'static str,
        request: &'static str,
        field: &'static str,
    },
}
