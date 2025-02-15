pub mod handlers;
pub mod server_channels;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum SpotArbScannerHttpRequests {
    TestRequest,
    TestResponse,
}
