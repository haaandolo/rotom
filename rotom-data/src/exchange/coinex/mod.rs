use async_trait::async_trait;
use channel::CoinExChannel;
use market::CoinExMarket;
use model::{CoinExNetworkInfo, CoinExOrderBookSnapshot, CoinExSubscriptionResponse, CoinExTrade};
use rand::Rng;
use serde_json::json;

use crate::{
    error::SocketError,
    model::{event_book_snapshot::OrderBookSnapshot, event_trade::Trades},
    protocols::ws::WsMessage,
    shared::subscription_models::{ExchangeId, ExchangeSubscription, Instrument},
    transformer::stateless_transformer::StatelessTransformer,
};

use super::{PublicHttpConnector, PublicStreamConnector, StreamSelector};

pub mod channel;
pub mod market;
pub mod model;

#[derive(Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Clone)]
pub struct CoinExSpotPublicData;

const COINEX_SPOT_WS_URL: &str = "wss://socket.coinex.com/v2/spot";

impl PublicStreamConnector for CoinExSpotPublicData {
    const ID: ExchangeId = ExchangeId::CoinExSpot;

    type Channel = CoinExChannel;
    type Market = CoinExMarket;
    type SubscriptionResponse = CoinExSubscriptionResponse;

    fn url() -> impl Into<String> {
        COINEX_SPOT_WS_URL
    }

    // Request for CoinEx is cooked. Add more if else statements here if you add more channels
    fn requests(
        subscriptions: &[ExchangeSubscription<Self, Self::Channel, Self::Market>],
    ) -> Option<WsMessage> {
        // I think you can only sub to one type of channel for [ExchangeSubscription] so this index is fine
        let channel = subscriptions[0].channel;
        let random_id = rand::thread_rng().gen::<u64>();

        // Trade channel
        if channel.0 == CoinExChannel::TRADES.0 {
            let params = subscriptions
                .iter()
                .map(|sub| sub.market.as_ref())
                .collect::<Vec<_>>();

            let request = json!({
                "method": channel.0,
                "params": {
                   "market_list": params
                },
                "id": random_id,
            });

            Some(WsMessage::text(request.to_string()))
        }
        // OrderBook snapshot channel
        else {
            let params = subscriptions
                .iter()
                .map(|sub| (sub.market.as_ref(), 10, "0", true))
                .collect::<Vec<(&str, u64, &str, bool)>>(); // Required by CoinEx in this format ref: https://docs.coinex.com/api/v2/spot/market/ws/market-depth

            let request = json!({
                "method": channel.0,
                "params": {
                   "market_list": params
                },
                "id": random_id,
            });

            Some(WsMessage::text(request.to_string()))
        }
    }

    fn expected_responses(
        _subscriptions: &[ExchangeSubscription<Self, Self::Channel, Self::Market>],
    ) -> usize {
        1
    }
}

/*----- */
// CoinExSpot HttpConnector
/*----- */
pub const COINEX_BASE_HTTP_URL: &str = "https://api.coinex.com/v2";

#[async_trait]
impl PublicHttpConnector for CoinExSpotPublicData {
    const ID: ExchangeId = ExchangeId::CoinExSpot;

    type BookSnapShot = serde_json::Value;
    type ExchangeTickerInfo = serde_json::Value;
    type NetworkInfo = CoinExNetworkInfo;

    async fn get_book_snapshot(_instrument: Instrument) -> Result<Self::BookSnapShot, SocketError> {
        unimplemented!()
    }

    async fn get_ticker_info(
        _instrument: Instrument,
    ) -> Result<Self::ExchangeTickerInfo, SocketError> {
        unimplemented!()
    }

    async fn get_network_info(
        _instruments: Vec<Instrument>,
    ) -> Result<Self::NetworkInfo, SocketError> {
        let request_path = "/assets/all-deposit-withdraw-config";
        Ok(
            reqwest::get(format!("{}{}", COINEX_BASE_HTTP_URL, request_path))
                .await
                .map_err(SocketError::Http)?
                .json::<Self::NetworkInfo>()
                .await
                .map_err(SocketError::Http)?,
        )
    }

    async fn get_usdt_pair() -> Result<Vec<(String, String)>, SocketError> {
        // Get tickers that meet volume threshold
        let request_path = "/spot/ticker";
        let response = reqwest::get(format!("{}{}", COINEX_BASE_HTTP_URL, request_path))
            .await
            .map_err(SocketError::Http)?
            .json::<serde_json::Value>()
            .await
            .map_err(SocketError::Http)?;

        let volume_threshold = Self::get_volume_threshold() as f64;
        let tickers = response["data"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|ticker| {
                let quote = String::from("usdt");
                let symbol = ticker["market"].as_str().unwrap().to_lowercase();
                let volume = ticker["volume"]
                    .as_str()
                    .unwrap_or("0.0")
                    .trim_matches('"') // This removes quotation marks
                    .parse::<f64>()
                    .unwrap_or(0.0);

                let last_price = ticker["last"]
                    .as_str()
                    .unwrap_or("0.0")
                    .trim_matches('"') // This removes quotation marks
                    .parse::<f64>()
                    .unwrap_or(0.0);

                let volume_in_usdt = volume * last_price;

                if symbol.contains(quote.as_str()) && volume_in_usdt > volume_threshold {
                    let base = symbol.replace(quote.as_str(), "");
                    Some((base, quote))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        Ok(tickers)
    }
}

/*----- */
// Stream selector
/*----- */
impl StreamSelector<CoinExSpotPublicData, OrderBookSnapshot> for CoinExSpotPublicData {
    type Stream = CoinExOrderBookSnapshot;
    type StreamTransformer =
        StatelessTransformer<CoinExSpotPublicData, Self::Stream, OrderBookSnapshot>;
}

impl StreamSelector<CoinExSpotPublicData, Trades> for CoinExSpotPublicData {
    type Stream = CoinExTrade;
    type StreamTransformer = StatelessTransformer<CoinExSpotPublicData, Self::Stream, Trades>;
}
