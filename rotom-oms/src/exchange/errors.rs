use rotom_data::error::SocketError;
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

impl From<RequestBuildError> for SocketError {
    fn from(error: RequestBuildError) -> Self {
        match error {
            RequestBuildError::BuilderError { exchange, request } => SocketError::RequestBuildError(format!(
                "RequestBuild::BuilderError encountered for {}. With request {}",
                exchange, request
            )),
            RequestBuildError::MandatoryField { exchange, request, field } => SocketError::RequestBuildError(format!(
                "RequestBuild::MandatoryField error encountered for {}. With request {} and missing field {}",
                exchange, request, field
            ))
        }
    }
}
