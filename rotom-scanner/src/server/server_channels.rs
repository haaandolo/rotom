use tokio::sync::mpsc;

use super::{SpotArbScannerHttpRequests, SpotArbScannerHttpResponse};

/*----- */
// Http Channels
/*----- */
#[derive(Debug)]
pub struct ScannerHttpChannel {
    pub http_request_rx: mpsc::UnboundedReceiver<SpotArbScannerHttpRequests>,
    pub http_response_tx: mpsc::UnboundedSender<SpotArbScannerHttpResponse>,
}

#[derive(Debug)]
pub struct ServerHttpChannel {
    pub http_request_tx: mpsc::UnboundedSender<SpotArbScannerHttpRequests>,
    pub http_response_rx: mpsc::UnboundedReceiver<SpotArbScannerHttpResponse>,
}

pub fn make_http_channels() -> (ScannerHttpChannel, ServerHttpChannel) {
    let (http_request_tx, http_request_rx) = mpsc::unbounded_channel();
    let (http_response_tx, http_response_rx) = mpsc::unbounded_channel();

    let scanner_http_channel = ScannerHttpChannel {
        http_request_rx,
        http_response_tx,
    };

    let server_http_channel = ServerHttpChannel {
        http_request_tx,
        http_response_rx,
    };

    (scanner_http_channel, server_http_channel)
}
