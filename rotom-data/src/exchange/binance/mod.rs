pub mod channel;
pub mod l2;
pub mod market;
pub mod model;

use async_trait::async_trait;
use channel::BinanceChannel;
use l2::BinanceSpotBookUpdater;
use market::BinanceMarket;
use model::{
    BinanceAggTrade, BinanceSpotBookUpdate, BinanceSpotSnapshot, BinanceSpotTickerInfo,
    BinanceSubscriptionResponse, BinanceTrade,
};
use serde_json::json;

use crate::{
    error::SocketError,
    exchange::{PublicHttpConnector, PublicStreamConnector, StreamSelector},
    model::{
        event_book::OrderBookL2,
        event_trade::{AggTrades, Trades},
    },
    protocols::ws::WsMessage,
    shared::subscription_models::{ExchangeId, ExchangeSubscription, Instrument},
    transformer::{book::MultiBookTransformer, stateless_transformer::StatelessTransformer},
};

const BINANCE_SPOT_WS_URL: &str = "wss://stream.binance.com:9443/ws";

/*----- */
// BinanceSpot connector
/*----- */
#[derive(Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Clone)]
pub struct BinanceSpotPublicData;

impl PublicStreamConnector for BinanceSpotPublicData {
    const ID: ExchangeId = ExchangeId::BinanceSpot;

    type SubscriptionResponse = BinanceSubscriptionResponse;
    type Channel = BinanceChannel;
    type Market = BinanceMarket;

    fn url() -> &'static str {
        BINANCE_SPOT_WS_URL
    }

    fn requests(
        subscriptions: &[ExchangeSubscription<Self, Self::Channel, Self::Market>],
    ) -> Option<WsMessage> {
        let binance_subs = subscriptions
            .iter()
            .map(|s| format!("{}{}", s.market.as_ref().to_lowercase(), s.channel.as_ref()))
            .collect::<Vec<_>>();

        let binance_request = json!({
            "method": "SUBSCRIBE",
            "params": binance_subs,
            "id": 1
        });

        Some(WsMessage::Text(binance_request.to_string()))
    }

    fn expected_responses(
        _subscriptions: &[ExchangeSubscription<Self, Self::Channel, Self::Market>],
    ) -> usize
    where
        Self: Sized,
    {
        1
    }
}

/*----- */
// Stream selector
/*----- */
impl StreamSelector<BinanceSpotPublicData, OrderBookL2> for BinanceSpotPublicData {
    type Stream = BinanceSpotBookUpdate;
    type StreamTransformer =
        MultiBookTransformer<BinanceSpotPublicData, BinanceSpotBookUpdater, OrderBookL2>;
}

impl StreamSelector<BinanceSpotPublicData, Trades> for BinanceSpotPublicData {
    type Stream = BinanceTrade;
    type StreamTransformer = StatelessTransformer<BinanceSpotPublicData, Self::Stream, Trades>;
}

impl StreamSelector<BinanceSpotPublicData, AggTrades> for BinanceSpotPublicData {
    type Stream = BinanceAggTrade;
    type StreamTransformer = StatelessTransformer<BinanceSpotPublicData, Self::Stream, AggTrades>;
}

/*----- */
// BinanceSpot HttpConnector
/*----- */
pub const HTTP_BOOK_L2_SNAPSHOT_URL_BINANCE_SPOT: &str = "https://api.binance.com/api/v3/depth";
pub const HTTP_TICKER_INFO_URL_BINANCE_SPOT: &str =
    "https://api.binance.us/api/v3/exchangeInfo?symbol=";

#[async_trait]
impl PublicHttpConnector for BinanceSpotPublicData {
    const ID: ExchangeId = ExchangeId::BinanceSpot;

    type BookSnapShot = BinanceSpotSnapshot;
    type ExchangeTickerInfo = BinanceSpotTickerInfo;
    type NetworkInfo = serde_json::Value; // todo

    async fn get_book_snapshot(instrument: Instrument) -> Result<Self::BookSnapShot, SocketError> {
        let snapshot_url = format!(
            "{}?symbol={}{}&limit=100",
            HTTP_BOOK_L2_SNAPSHOT_URL_BINANCE_SPOT,
            instrument.base.to_uppercase(),
            instrument.quote.to_uppercase()
        );

        reqwest::get(snapshot_url)
            .await
            .map_err(SocketError::Http)?
            .json::<BinanceSpotSnapshot>()
            .await
            .map_err(SocketError::Http)
    }

    async fn get_ticker_info(
        instrument: Instrument,
    ) -> Result<Self::ExchangeTickerInfo, SocketError> {
        let ticker_info_url = format!(
            "{}{}{}",
            HTTP_TICKER_INFO_URL_BINANCE_SPOT,
            instrument.base.to_uppercase(),
            instrument.quote.to_uppercase()
        );

        reqwest::get(ticker_info_url)
            .await
            .map_err(SocketError::Http)?
            .json::<BinanceSpotTickerInfo>()
            .await
            .map_err(SocketError::Http)
    }

    async fn get_network_info() -> Result<Self::NetworkInfo, SocketError> {
        unimplemented!()
    }
}
