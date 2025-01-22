pub mod channel;
pub mod market;
pub mod model;

use async_trait::async_trait;
use channel::HtxChannel;
use market::HtxMarket;
use model::{HtxOrderBookSnapshot, HtxSubscriptionResponse, HtxTrade, HtxWalletInfo};
use rand::Rng;
use serde_json::json;

use crate::{
    error::SocketError,
    model::{event_book_snapshot::OrderBookSnapshot, event_trade::TradesVec},
    protocols::ws::{PingInterval, WsMessage},
    shared::subscription_models::{ExchangeId, ExchangeSubscription, Instrument},
    transformer::stateless_transformer::StatelessTransformer,
};

use super::{PublicHttpConnector, PublicStreamConnector, StreamSelector};

const HTX_SPOT_WS_URL: &str = "wss://api-aws.huobi.pro/ws";

#[derive(Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Clone)]
pub struct HtxSpotPublicData;

impl PublicStreamConnector for HtxSpotPublicData {
    const ID: ExchangeId = ExchangeId::HtxSpot;

    type Channel = HtxChannel;
    type Market = HtxMarket;
    type SubscriptionResponse = HtxSubscriptionResponse;

    fn url() -> &'static str {
        HTX_SPOT_WS_URL
    }

    fn requests(
        subscriptions: &[ExchangeSubscription<Self, Self::Channel, Self::Market>],
    ) -> Option<WsMessage> {
        let htx_subs = subscriptions
            .iter()
            .map(|s| format!("market.{}.{}", s.market.as_ref(), s.channel.as_ref()))
            .collect::<Vec<_>>();

        let htx_request = json!({
            "sub": htx_subs,
            "id": rand::thread_rng().gen::<u64>().to_string(),
        });

        Some(WsMessage::text(htx_request.to_string()))
    }

    fn ping_interval() -> Option<PingInterval> {
        Some(PingInterval {
            time: 4,
            message: json!({ "pong": rand::thread_rng().gen::<u64>() }),
        })
    }
}

/*----- */
// HtxSpot HttpConnector
/*----- */
pub const HTTP_CHAIN_INFO_URL_HTX_SPOT: &str =
    "https://api-aws.huobi.pro/v1/settings/common/chains";

#[async_trait]
impl PublicHttpConnector for HtxSpotPublicData {
    const ID: ExchangeId = ExchangeId::HtxSpot;

    type BookSnapShot = serde_json::Value;
    type ExchangeTickerInfo = serde_json::Value;
    type WalletInfo = HtxWalletInfo; // todo

    async fn get_book_snapshot(_instrument: Instrument) -> Result<Self::BookSnapShot, SocketError> {
        unimplemented!()
    }

    async fn get_ticker_info(
        _instrument: Instrument,
    ) -> Result<Self::ExchangeTickerInfo, SocketError> {
        unimplemented!()
    }

    async fn get_chain_info() -> Result<Self::WalletInfo, SocketError> {
        Ok(reqwest::get(HTTP_CHAIN_INFO_URL_HTX_SPOT)
            .await
            .map_err(SocketError::Http)?
            .json::<Self::WalletInfo>()
            .await
            .map_err(SocketError::Http)?)
    }
}

/*----- */
// Stream selector
/*----- */
impl StreamSelector<HtxSpotPublicData, OrderBookSnapshot> for HtxSpotPublicData {
    type Stream = HtxOrderBookSnapshot;
    type StreamTransformer =
        StatelessTransformer<HtxSpotPublicData, Self::Stream, OrderBookSnapshot>;
}

impl StreamSelector<HtxSpotPublicData, TradesVec> for HtxSpotPublicData {
    type Stream = HtxTrade;
    type StreamTransformer = StatelessTransformer<HtxSpotPublicData, Self::Stream, TradesVec>;
}
