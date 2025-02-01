pub mod channel;
pub mod l2;
pub mod market;
pub mod model;

use async_trait::async_trait;
use l2::PoloniexSpotBookUpdater;
use market::PoloniexMarket;
use serde_json::json;

use crate::error::SocketError;
use crate::exchange::{PublicHttpConnector, PublicStreamConnector, StreamSelector};
use crate::model::event_book::OrderBookL2;
use crate::model::event_trade::Trade;
use crate::protocols::ws::{PingInterval, WsMessage};
use crate::shared::subscription_models::{ExchangeId, ExchangeSubscription, Instrument};
use crate::transformer::book::MultiBookTransformer;
use crate::transformer::stateless_transformer::StatelessTransformer;
use channel::PoloniexChannel;
use model::{
    PoloniexSpotBookUpdate, PoloniexSpotTickerInfo, PoloniexSubscriptionResponse, PoloniexTrade,
};

const POLONIEX_SPOT_WS_URL: &str = "wss://ws.poloniex.com/ws/public";

/*----- */
// PoloniexSpot connector
/*----- */
#[derive(Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Clone)]
pub struct PoloniexSpotPublicData;

impl PublicStreamConnector for PoloniexSpotPublicData {
    const ID: ExchangeId = ExchangeId::PoloniexSpot;

    type SubscriptionResponse = PoloniexSubscriptionResponse;
    type Channel = PoloniexChannel;
    type Market = PoloniexMarket;

    fn url() -> impl Into<String> {
        POLONIEX_SPOT_WS_URL
    }

    fn requests(
        sub: &[ExchangeSubscription<Self, Self::Channel, Self::Market>],
    ) -> Option<WsMessage> {
        let channels = sub.iter().map(|s| s.channel.as_ref()).collect::<Vec<_>>();
        let tickers = sub.iter().map(|s| s.market.as_ref()).collect::<Vec<_>>();
        let poloniex_sub = json!({
            "event": "subscribe",
            "channel": channels,
            "symbols": tickers
        });
        Some(WsMessage::text(poloniex_sub.to_string()))
    }

    fn ping_interval() -> Option<PingInterval> {
        Some(PingInterval {
            time: 25,
            message: json!({"event": "ping"}),
        })
    }
}

/*----- */
// Stream selector
/*----- */
impl StreamSelector<PoloniexSpotPublicData, OrderBookL2> for PoloniexSpotPublicData {
    type Stream = PoloniexSpotBookUpdate;
    type StreamTransformer =
        MultiBookTransformer<PoloniexSpotPublicData, PoloniexSpotBookUpdater, OrderBookL2>;
}

impl StreamSelector<PoloniexSpotPublicData, Trade> for PoloniexSpotPublicData {
    type Stream = PoloniexTrade;
    type StreamTransformer = StatelessTransformer<PoloniexSpotPublicData, Self::Stream, Trade>;
}

/*----- */
// PoloniexSpot HttpConnector
/*----- */
pub const HTTP_TICKER_INFO_URL_POLONIEX_SPOT: &str = "https://api.poloniex.com/markets/";

#[async_trait]
impl PublicHttpConnector for PoloniexSpotPublicData {
    const ID: ExchangeId = ExchangeId::PoloniexSpot;

    type BookSnapShot = serde_json::Value;
    type ExchangeTickerInfo = PoloniexSpotTickerInfo;
    type NetworkInfo = serde_json::Value; // todo

    // We dont need this for Poloniex rn as snapshot comes through stream. This function should NEVER be called
    async fn get_book_snapshot(_instrument: Instrument) -> Result<Self::BookSnapShot, SocketError> {
        todo!()
    }

    // This function returns a Vec<PoloniexSpotTickerInfo> but the function only
    // takes in a single instrument, we should only get a Vec of len == 1. So
    // here we can index by 0 i.e. info[0] to get the res.
    async fn get_ticker_info(
        instrument: Instrument,
    ) -> Result<Self::ExchangeTickerInfo, SocketError> {
        let ticker_info_url = format!(
            "{}{}_{}",
            HTTP_TICKER_INFO_URL_POLONIEX_SPOT,
            instrument.base.to_uppercase(),
            instrument.quote.to_uppercase()
        );

        Ok(reqwest::get(ticker_info_url)
            .await
            .map_err(SocketError::Http)?
            .json::<Vec<PoloniexSpotTickerInfo>>()
            .await
            .map_err(SocketError::Http)?
            .remove(0))
    }

    async fn get_network_info(
        _instruments: Vec<Instrument>,
    ) -> Result<Self::NetworkInfo, SocketError> {
        unimplemented!()
    }
}
