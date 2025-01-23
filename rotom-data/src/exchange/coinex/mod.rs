use channel::CoinExChannel;
use market::CoinExMarket;
use model::{CoinExOrderBookSnapshot, CoinExSubscriptionResponse};
use rand::Rng;
use serde_json::json;

use crate::{
    model::event_book_snapshot::OrderBookSnapshot,
    protocols::ws::WsMessage,
    shared::subscription_models::{ExchangeId, ExchangeSubscription},
    transformer::stateless_transformer::StatelessTransformer,
};

use super::{PublicStreamConnector, StreamSelector};

pub mod channel;
pub mod market;
pub mod model;

#[derive(Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Clone)]
pub struct CoinExSpotPublicData;

const COINEX_SPOT_WS_URL: &str = "wss://socket.coinex.com/v2/spot";

impl PublicStreamConnector for CoinExSpotPublicData {
    const ID: ExchangeId = ExchangeId::CoinEx;

    type Channel = CoinExChannel;
    type Market = CoinExMarket;
    type SubscriptionResponse = CoinExSubscriptionResponse;

    fn url() -> &'static str {
        COINEX_SPOT_WS_URL
    }

    // Request for CoinEx is cooked, add more if else statements here if you add more channels
    fn requests(
        subscriptions: &[ExchangeSubscription<Self, Self::Channel, Self::Market>],
    ) -> Option<WsMessage> {
        // I think you can only sub to one type of channel for [ExchangeSubscription] so this index is fine
        let channel = subscriptions[0].channel;

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
                "id": rand::thread_rng().gen::<u64>(),
            });

            println!("request: \n {:#?}", request);

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
                // "id": rand::thread_rng().gen::<u64>(),
                "id": 1
            });

            Some(WsMessage::text(request.to_string()))
        }
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

// impl StreamSelector<CoinExSpotPublicData, Trades> for CoinExSpotPublicData {
//     type Stream = CoinExOrderBookSnapshot;
//     type StreamTransformer = StatelessTransformer<CoinExSpotPublicData, Self::Stream, Trades>;
// }
