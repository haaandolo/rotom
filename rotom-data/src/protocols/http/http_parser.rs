use crate::error::SocketError;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use tracing::error;

pub trait HttpParser {
    type ApiError: DeserializeOwned;
    type OutputError: From<SocketError>;

    fn parse<Response>(
        &self,
        status: StatusCode,
        payload: &[u8],
    ) -> Result<Response, Self::OutputError>
    where
        Response: DeserializeOwned,
    {
        // Attempt to deserialise reqwest::Response bytes into Ok(Response)
        let parse_ok_error = match serde_json::from_slice::<Response>(payload) {
            Ok(response) => return Ok(response),
            Err(serde_error) => serde_error,
        };

        // Attempt to deserialise API Error if Ok(Response) deserialisation failed
        let parse_api_error_error = match serde_json::from_slice::<Self::ApiError>(payload) {
            Ok(api_error) => return Err(self.parse_api_error(status, api_error)),
            Err(serde_error) => serde_error,
        };

        // Log errors if failed to deserialise reqwest::Response into Response or API Self::Error
        error!(
            status_code = ?status,
            ?parse_ok_error,
            ?parse_api_error_error,
            response_body = %String::from_utf8_lossy(payload),
            "error deserializing HTTP response"
        );

        Err(Self::OutputError::from(SocketError::DeserialiseBinary {
            error: parse_ok_error,
            payload: payload.to_vec(),
        }))
    }

    // If [`parse`](Self::parse) fails to deserialise the `Ok(Response)`, this function parses
    // to parse the API [`Self::ApiError`] associated with the response.
    fn parse_api_error(&self, status: StatusCode, error: Self::ApiError) -> Self::OutputError;
}

#[derive(Debug)]
pub struct StandardHttpParser;

impl HttpParser for StandardHttpParser {
    type ApiError = serde_json::Value;
    type OutputError = SocketError;

    fn parse_api_error(&self, status: StatusCode, api_error: Self::ApiError) -> Self::OutputError {
        // For simplicity, use serde_json::Value as Error and extract raw String for parsing
        let error = api_error.to_string();

        // Parse Ftx error message to determine custom ExecutionError variant
        match error.as_str() {
            message if message.contains("Invalid login credentials") => {
                SocketError::Unauthorised(error)
            }
            _ => SocketError::HttpResponse(status, error),
        }
    }
}
