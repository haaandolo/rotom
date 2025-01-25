pub mod channel;
pub mod market;
pub mod model;

use channel::KuCoinChannel;
use market::KuCoinMarket;
use model::{KuCoinOrderBookSnapshot, KuCoinSubscriptionResponse, KuCoinTrade, KuCoinWsUrl};
use serde_json::json;

use crate::{
    error::SocketError,
    model::{event_book_snapshot::OrderBookSnapshot, event_trade::Trades},
    protocols::ws::{PingInterval, WsMessage},
    shared::subscription_models::{ExchangeId, ExchangeSubscription},
    transformer::stateless_transformer::StatelessTransformer,
};

use super::{PublicStreamConnector, StreamSelector};

#[derive(Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Clone)]
pub struct KuCoinSpotPublicData;

impl PublicStreamConnector for KuCoinSpotPublicData {
    const ID: ExchangeId = ExchangeId::KuCoinSpot;

    type Channel = KuCoinChannel;
    type Market = KuCoinMarket;
    type SubscriptionResponse = KuCoinSubscriptionResponse;

    fn url() -> impl Into<String> {
        let base_url_http = "https://api.kucoin.com";
        let token_post = "/api/v1/bullet-public";

        let kucoin_ws_url_response = tokio::task::block_in_place(|| {
            reqwest::blocking::Client::new()
                .post(format!("{}{}", base_url_http, token_post))
                .send()
                .map_err(SocketError::Http)
                .unwrap()
                .json::<KuCoinWsUrl>()
                .map_err(SocketError::Http)
                .unwrap()
        });

        format!(
            "{}?token={}&[connectId={}]",
            kucoin_ws_url_response.data.instance_servers[0].endpoint,
            kucoin_ws_url_response.data.token,
            uuid::Uuid::new_v4()
        )
    }

    fn requests(
        subscriptions: &[ExchangeSubscription<Self, Self::Channel, Self::Market>],
    ) -> Option<WsMessage> {
        let channel = subscriptions[0].channel; // Each vec of subs can be of one channel type so this works
        let subs = subscriptions
            .iter()
            .map(|s| s.market.as_ref())
            .collect::<Vec<_>>()
            .join(",");

        let sub_request = format!("{}{}", channel.0, subs);
        let request = json!({
            "id": uuid::Uuid::new_v4(),
            "type": "subscribe",
            "topic": sub_request,
            "response": true
        });

        println!("request: {:#?}", request);

        Some(WsMessage::text(request.to_string()))
    }

    fn expected_responses(
        _subscriptions: &[ExchangeSubscription<Self, Self::Channel, Self::Market>],
    ) -> usize {
        // 2 because first response is a message stating the ws connected succesfully.
        // The second is the success message for subbing to channels
        2
    }

    fn ping_interval() -> Option<PingInterval> {
        Some(PingInterval {
            time: 30,
            message: json!({ "id":  uuid::Uuid::new_v4(), "type": "ping"}),
        })
    }
}

/*----- */
// Stream selector
/*----- */
impl StreamSelector<KuCoinSpotPublicData, OrderBookSnapshot> for KuCoinSpotPublicData {
    type Stream = KuCoinOrderBookSnapshot;
    type StreamTransformer =
        StatelessTransformer<KuCoinSpotPublicData, Self::Stream, OrderBookSnapshot>;
}

impl StreamSelector<KuCoinSpotPublicData, Trades> for KuCoinSpotPublicData {
    type Stream = KuCoinTrade;
    type StreamTransformer = StatelessTransformer<KuCoinSpotPublicData, Self::Stream, Trades>;
}
