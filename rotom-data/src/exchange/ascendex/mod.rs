use async_trait::async_trait;
use channel::AscendExChannel;
use l2::AscendExSpotBookUpdater;
use market::AscendExMarket;
use model::{
    AscendExBookUpdate, AscendExOrderBookSnapshot, AscendExSubscriptionResponse,
    AscendExTickerInfo, AscendExTrades,
};
use serde_json::json;

use crate::{
    error::SocketError,
    model::{event_book::OrderBookL2, event_trade::Trades},
    protocols::ws::WsMessage,
    shared::subscription_models::{ExchangeId, ExchangeSubscription, Instrument},
    transformer::{book::MultiBookTransformer, stateless_transformer::StatelessTransformer},
};

use super::{PublicHttpConnector, PublicStreamConnector, StreamSelector};

pub mod channel;
pub mod l2;
pub mod market;
pub mod model;

const ASCENDEX_SPOT_WS_URL: &str = "wss://ascendex.com/7/api/pro/v1/stream";

/*----- */
// Stream Connector
/*----- */
#[derive(Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Clone)]
pub struct AscendExSpotPublicData;

impl PublicStreamConnector for AscendExSpotPublicData {
    const ID: ExchangeId = ExchangeId::AscendExSpot;

    type Channel = AscendExChannel;
    type Market = AscendExMarket;
    type SubscriptionResponse = AscendExSubscriptionResponse;

    fn url() -> impl Into<String> {
        ASCENDEX_SPOT_WS_URL
    }

    // Request for CoinEx is cooked. Add more if else statements here if you add more channels
    fn requests(
        subscriptions: &[ExchangeSubscription<Self, Self::Channel, Self::Market>],
    ) -> Option<WsMessage> {
        // I think you can only sub to one type of channel for [ExchangeSubscription] so this index is fine
        let channel = &subscriptions[0].channel;

        let subs = subscriptions
            .iter()
            .map(|s| s.market.as_ref())
            .collect::<Vec<&str>>()
            .join(",");

        let request_param = format!("{}{}", channel.0, subs);

        let request = json!({
            "op": "sub",
            "id": uuid::Uuid::new_v4(),
            "ch": request_param
        });

        println!("request: {}", request);

        Some(WsMessage::text(request.to_string()))
    }
}

/*----- */
// HttpConnector
/*----- */
pub const ASCENDEX_BASE_HTTP_URL: &str = "https://ascendex.com";

#[async_trait]
impl PublicHttpConnector for AscendExSpotPublicData {
    const ID: ExchangeId = ExchangeId::AscendExSpot;

    type BookSnapShot = AscendExOrderBookSnapshot;
    type ExchangeTickerInfo = AscendExTickerInfo;
    type NetworkInfo = serde_json::Value;

    async fn get_book_snapshot(instrument: Instrument) -> Result<Self::BookSnapShot, SocketError> {
        let request_path = "/api/pro/v1/depth";
        let snapshot_url = format!(
            "{}{}?symbol={}/{}",
            ASCENDEX_BASE_HTTP_URL,
            request_path,
            instrument.base.to_uppercase(),
            instrument.quote.to_uppercase()
        );

        reqwest::get(snapshot_url)
            .await
            .map_err(SocketError::Http)?
            .json::<Self::BookSnapShot>()
            .await
            .map_err(SocketError::Http)
    }

    async fn get_ticker_info(
        _instrument: Instrument,
    ) -> Result<Self::ExchangeTickerInfo, SocketError> {
        let request_path = "/api/pro/v1/cash/products";
        let ticker_info_url = format!("{}{}", ASCENDEX_BASE_HTTP_URL, request_path,);

        reqwest::get(ticker_info_url)
            .await
            .map_err(SocketError::Http)?
            .json::<Self::ExchangeTickerInfo>()
            .await
            .map_err(SocketError::Http)
    }

    async fn get_network_info() -> Result<Self::NetworkInfo, SocketError> {
        unimplemented!()
    }
}

/*----- */
// Stream selector
/*----- */
impl StreamSelector<AscendExSpotPublicData, OrderBookL2> for AscendExSpotPublicData {
    type Stream = AscendExBookUpdate;
    type StreamTransformer =
        MultiBookTransformer<AscendExSpotPublicData, AscendExSpotBookUpdater, OrderBookL2>;
}

impl StreamSelector<AscendExSpotPublicData, Trades> for AscendExSpotPublicData {
    type Stream = AscendExTrades;
    type StreamTransformer = StatelessTransformer<AscendExSpotPublicData, Self::Stream, Trades>;
}
