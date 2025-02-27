use actix_web::{
    get,
    web::{self, Data},
    HttpResponse, Responder,
};
use rotom_data::shared::de::de_lowercase;
use rotom_data::shared::subscription_models::{ExchangeId, Instrument};
use serde::Deserialize;
use tokio::{
    sync::Mutex,
    time::{timeout, Duration},
};

use crate::server::{server_channels::ServerHttpChannel, SpotArbScannerHttpRequests};

/*----- */
// Handler
/*----- */
#[get("/")]
pub async fn get_top_spreads_handler(
    channel_data: Data<Mutex<ServerHttpChannel>>,
) -> impl Responder {
    let mut http_channel = channel_data.lock().await;

    // Send the request
    if http_channel
        .http_request_tx
        .send(SpotArbScannerHttpRequests::GetTopSpreads)
        .is_err()
    {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to send request to scanner"
        }));
    }

    // Wait for response with timeout
    match timeout(
        Duration::from_millis(100),
        http_channel.http_response_rx.recv(),
    )
    .await
    {
        Ok(Some(response)) => HttpResponse::Ok().json(serde_json::to_value(&response).unwrap()),
        Ok(None) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Scanner channel closed unexpectedly"
        })),
        Err(_) => HttpResponse::RequestTimeout().json(serde_json::json!({
            "error": "Request timed out after 5 seconds"
        })),
    }
}

#[get("/ws-status")]
pub async fn get_ws_connection_status_handler(
    channel_data: Data<Mutex<ServerHttpChannel>>,
) -> impl Responder {
    let mut http_channel = channel_data.lock().await;

    // Send the request
    if http_channel
        .http_request_tx
        .send(SpotArbScannerHttpRequests::GetWsConnectionStatus)
        .is_err()
    {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to send request to scanner"
        }));
    }

    // Wait for response with timeout
    match timeout(
        Duration::from_millis(100),
        http_channel.http_response_rx.recv(),
    )
    .await
    {
        Ok(Some(response)) => HttpResponse::Ok().json(serde_json::to_value(&response).unwrap()),
        Ok(None) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Scanner channel closed unexpectedly"
        })),
        Err(_) => HttpResponse::RequestTimeout().json(serde_json::json!({
            "error": "Request timed out after 5 seconds"
        })),
    }
}

#[derive(Deserialize)]
// #[serde(rename_all = "snake_case")]
struct SpreadHistoryQueryParams {
    base_exchange: ExchangeId,
    quote_exchange: ExchangeId,
    #[serde(deserialize_with = "de_lowercase")]
    base_instrument: String,
    #[serde(deserialize_with = "de_lowercase")]
    quote_instrument: String,
}

#[get("spread-history")]
pub async fn get_spread_history_handler(
    channel_data: Data<Mutex<ServerHttpChannel>>,
    query: web::Query<SpreadHistoryQueryParams>,
) -> impl Responder {
    let mut http_channel = channel_data.lock().await;

    let base_exchange = query.base_exchange;
    let quote_exchange = query.quote_exchange;
    let instrument = Instrument::new(
        query.base_instrument.clone(),
        query.quote_instrument.clone(),
    );

    // Send the request
    if http_channel
        .http_request_tx
        .send(SpotArbScannerHttpRequests::GetSpreadHistory((
            base_exchange,
            quote_exchange,
            instrument,
        )))
        .is_err()
    {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to send request to scanner"
        }));
    }

    // Wait for response with timeout
    match timeout(
        Duration::from_millis(100),
        http_channel.http_response_rx.recv(),
    )
    .await
    {
        Ok(Some(response)) => HttpResponse::Ok().json(serde_json::to_value(&response).unwrap()),
        Ok(None) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Scanner channel closed unexpectedly"
        })),
        Err(_) => HttpResponse::RequestTimeout().json(serde_json::json!({
            "error": "Request timed out after 5 seconds"
        })),
    }
}
