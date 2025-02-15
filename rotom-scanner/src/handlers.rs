use actix_web::{get, web::Data, HttpResponse, Responder};
use tokio::{
    sync::{mpsc, Mutex},
    time::{timeout, Duration},
};

use crate::scanner::SpotArbScannerHttpRequests;

/*----- */
// Http Channels
/*----- */
#[derive(Debug)]
pub struct ScannerHttpChannel {
    pub http_request_rx: mpsc::Receiver<SpotArbScannerHttpRequests>,
    pub http_response_tx: mpsc::Sender<SpotArbScannerHttpRequests>,
}

#[derive(Debug)]
pub struct ServerHttpChannel {
    pub http_request_tx: mpsc::Sender<SpotArbScannerHttpRequests>,
    pub http_response_rx: mpsc::Receiver<SpotArbScannerHttpRequests>,
}

pub fn make_http_channels() -> (ScannerHttpChannel, ServerHttpChannel) {
    let (http_request_tx, http_request_rx) = mpsc::channel(32);
    let (http_response_tx, http_response_rx) = mpsc::channel(32);

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

/*----- */
// Handler
/*----- */
#[get("/")]
pub async fn handler(channel_data: Data<Mutex<ServerHttpChannel>>) -> impl Responder {
    let mut data = channel_data.lock().await;

    // Send the request
    if data
        .http_request_tx
        .send(SpotArbScannerHttpRequests::TestRequest)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to send request to scanner"
        }));
    }

    // Wait for response with timeout
    match timeout(Duration::from_secs(5), data.http_response_rx.recv()).await {
        Ok(Some(response)) => HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "data": format!("{:?}", response)
        })),
        Ok(None) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Scanner channel closed unexpectedly"
        })),
        Err(_) => HttpResponse::RequestTimeout().json(serde_json::json!({
            "error": "Request timed out after 5 seconds"
        })),
    }
}
