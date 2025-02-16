use actix_web::{get, web::Data, HttpResponse, Responder};
use tokio::{
    sync::Mutex,
    time::{timeout, Duration},
};

use crate::server::{server_channels::ServerHttpChannel, SpotArbScannerHttpRequests};

/*----- */
// Handler
/*----- */
#[get("/")]
pub async fn handler(channel_data: Data<Mutex<ServerHttpChannel>>) -> impl Responder {
    let mut data = channel_data.lock().await;

    // Send the request
    if data
        .http_request_tx
        .send(SpotArbScannerHttpRequests::GetTopSpreads)
        .is_err()
    {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to send request to scanner"
        }));
    }

    // Wait for response with timeout
    match timeout(Duration::from_secs(5), data.http_response_rx.recv()).await {
        Ok(Some(response)) => HttpResponse::Ok().json(serde_json::to_value(&response).unwrap()),
        Ok(None) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Scanner channel closed unexpectedly"
        })),
        Err(_) => HttpResponse::RequestTimeout().json(serde_json::json!({
            "error": "Request timed out after 5 seconds"
        })),
    }
}
