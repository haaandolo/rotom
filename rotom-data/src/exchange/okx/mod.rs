pub mod channel;
pub mod market;
pub mod model;

use async_trait::async_trait;
use base64::Engine;
use channel::OkxChannel;
use chrono::Utc;
use hmac::{Hmac, Mac};
use market::OkxMarket;
use model::{OkxNetworkInfo, OkxOrderBookSnapshot, OkxSubscriptionResponse, OkxTrade};
use serde_json::json;
use sha2::Sha256;

use crate::{
    error::SocketError,
    model::{event_book_snapshot::OrderBookSnapshot, event_trade::Trade},
    protocols::ws::WsMessage,
    shared::subscription_models::{ExchangeId, ExchangeSubscription, Instrument},
    transformer::stateless_transformer::StatelessTransformer,
};

use super::{PublicHttpConnector, PublicStreamConnector, StreamSelector};
#[derive(Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Clone)]
pub struct OkxSpotPublicData;

const OKX_SPOT_WS_URL: &str = "wss://wspap.okx.com:8443/ws/v5/public";

impl PublicStreamConnector for OkxSpotPublicData {
    const ID: ExchangeId = ExchangeId::OkxSpot;

    type Channel = OkxChannel;
    type Market = OkxMarket;
    type SubscriptionResponse = OkxSubscriptionResponse;

    fn url() -> impl Into<String> {
        OKX_SPOT_WS_URL
    }

    fn requests(
        subscriptions: &[ExchangeSubscription<Self, Self::Channel, Self::Market>],
    ) -> Option<WsMessage> {
        let subs = subscriptions
            .iter()
            .map(|s| json!({"channel": s.channel.as_ref(), "instId": s.market.as_ref()}))
            .collect::<Vec<_>>();

        let request = json!({
            "op": "subscribe",
            "args": subs
        });

        Some(WsMessage::text(request.to_string()))
    }
}

/*----- */
// Okx HttpConnector
/*----- */
pub const OKX_BASE_HTTP_URL: &str = "https://www.okx.com";

#[async_trait]
impl PublicHttpConnector for OkxSpotPublicData {
    const ID: ExchangeId = ExchangeId::HtxSpot;

    type BookSnapShot = serde_json::Value;
    type ExchangeTickerInfo = serde_json::Value;
    type NetworkInfo = OkxNetworkInfo;

    async fn get_book_snapshot(_instrument: Instrument) -> Result<Self::BookSnapShot, SocketError> {
        unimplemented!()
    }

    async fn get_ticker_info(
        _instrument: Instrument,
    ) -> Result<Self::ExchangeTickerInfo, SocketError> {
        unimplemented!()
    }

    async fn get_network_info() -> Result<Self::NetworkInfo, SocketError> {
        // Import secrets, keys, passpharase etc
        let secret = env!("OKX_API_SECRET");
        let key = env!("OKX_API_KEY");
        let passphrase = env!("OKX_PASSPHRASE");

        // Define request params
        let timestamp = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let method = "GET";
        let request_path = "/api/v5/asset/currencies";
        let sign_message = format!("{}{}{}", timestamp, method, request_path);

        // Sign message
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(sign_message.as_bytes());
        let signature =
            base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());

        // Send request
        Ok(reqwest::Client::new()
            .get(format!("{}{}", OKX_BASE_HTTP_URL, request_path))
            .header("OK-ACCESS-KEY", key)
            .header("OK-ACCESS-SIGN", signature)
            .header("OK-ACCESS-TIMESTAMP", timestamp)
            .header("OK-ACCESS-PASSPHRASE", passphrase)
            .send()
            .await
            .map_err(SocketError::Http)?
            .json::<Self::NetworkInfo>()
            .await
            .map_err(SocketError::Http)?)
    }
}

/*----- */
// Stream selector
/*----- */
impl StreamSelector<OkxSpotPublicData, OrderBookSnapshot> for OkxSpotPublicData {
    type Stream = OkxOrderBookSnapshot;
    type StreamTransformer =
        StatelessTransformer<OkxSpotPublicData, Self::Stream, OrderBookSnapshot>;
}

impl StreamSelector<OkxSpotPublicData, Trade> for OkxSpotPublicData {
    type Stream = OkxTrade;
    type StreamTransformer = StatelessTransformer<OkxSpotPublicData, Self::Stream, Trade>;
}
