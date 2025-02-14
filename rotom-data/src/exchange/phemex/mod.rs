pub mod channel;
pub mod l2;
pub mod market;
pub mod model;

use async_trait::async_trait;
use channel::PhemexChannel;
use chrono::Utc;
use futures::try_join;
use hmac::{Hmac, Mac};
use l2::PhemexSpotBookUpdater;
use market::PhemexMarket;
use model::{
    PhemexDeposit, PhemexDepositData, PhemexOrderBookUpdate, PhemexSubscriptionResponse,
    PhemexTickerInfo, PhemexTradesUpdate, PhemexWithdraw, PhemexWithdrawChainInfo,
};
use rand::Rng;
use serde::Deserialize;
use serde_json::json;
use sha2::Sha256;
use std::collections::HashMap;

use crate::{
    error::SocketError,
    model::{
        event_book::OrderBookL2,
        event_trade::Trades,
        network_info::{ChainSpecs, NetworkSpecData, NetworkSpecs},
    },
    protocols::ws::{PingInterval, WsMessage},
    shared::subscription_models::{Coin, ExchangeId, ExchangeSubscription, Instrument},
    transformer::{book::MultiBookTransformer, stateless_transformer::StatelessTransformer},
};

use super::{PublicHttpConnector, PublicStreamConnector, StreamSelector};

#[derive(Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Clone)]
pub struct PhemexSpotPublicData;

/*----- */
// Stream connector
/*----- */
const PHEMEX_SPOT_WS_URL: &str = "wss://ws.phemex.com";

impl PublicStreamConnector for PhemexSpotPublicData {
    const ID: ExchangeId = ExchangeId::PhemexSpot;

    type SubscriptionResponse = PhemexSubscriptionResponse;
    type Channel = PhemexChannel;
    type Market = PhemexMarket;

    fn url() -> impl Into<String> {
        PHEMEX_SPOT_WS_URL
    }

    // Note: Phemex can only have one ticker per connection
    fn requests(
        subscriptions: &[ExchangeSubscription<Self, Self::Channel, Self::Market>],
    ) -> Option<WsMessage> {
        let channel = &subscriptions[0].channel; // One channel type per Vec<ExchangeSubscription>

        let subs = subscriptions
            .iter()
            .map(|s| s.market.as_ref())
            .collect::<Vec<&str>>();

        let request = json!({
            "id": rand::thread_rng().gen::<u64>(),
            "method": channel.0,
            "params": subs,
        });

        Some(WsMessage::Text(request.to_string()))
    }

    fn ping_interval() -> Option<PingInterval> {
        Some(PingInterval {
            time: 25,
            message: json!({"event": "ping"}),
        })
    }
}

/*----- */
// HttpConnector
/*----- */
pub const PHEMEX_BASE_HTTP_URL: &str = "https://api.phemex.com";

#[async_trait]
impl PublicHttpConnector for PhemexSpotPublicData {
    const ID: ExchangeId = ExchangeId::PhemexSpot;

    type BookSnapShot = serde_json::Value;
    type ExchangeTickerInfo = PhemexTickerInfo;
    type NetworkInfo = NetworkSpecs;

    async fn get_book_snapshot(_instrument: Instrument) -> Result<Self::BookSnapShot, SocketError> {
        unimplemented!()
    }

    async fn get_ticker_info(
        _instrument: Instrument,
    ) -> Result<Self::ExchangeTickerInfo, SocketError> {
        let request_path = "/public/products";
        let ticker_info_url = format!("{}{}", PHEMEX_BASE_HTTP_URL, request_path);

        reqwest::get(ticker_info_url)
            .await
            .map_err(SocketError::Http)?
            .json::<Self::ExchangeTickerInfo>()
            .await
            .map_err(SocketError::Http)
    }

    async fn get_network_info(
        instruments: Vec<Instrument>,
    ) -> Result<Self::NetworkInfo, SocketError> {
        let secret = env!("PHEMEX_API_SECRET");
        let key = env!("PHEMEX_API_KEY");

        let client = reqwest::Client::new();
        let request_path_withdraw = "/phemex-withdraw/wallets/api/asset/info";
        let request_path_deposit = "/phemex-deposit/wallets/api/chainCfg";
        let expiry = Utc::now().timestamp() as u64 + 60;
        let mut network_specs = HashMap::new();

        for instrument in instruments.into_iter() {
            let coin = instrument.base.to_uppercase();
            let query = format!("currency={}", &coin);

            // Make withdraw future
            let withdraw_message = format!("{}{}{}", request_path_withdraw, &query, expiry);
            let signature = sign_message_phemex(withdraw_message, secret);

            let withdraw_url = format!(
                "{}{}?{}",
                PHEMEX_BASE_HTTP_URL, request_path_withdraw, &query,
            );

            let withdraw_future = get_network_info_phemex::<PhemexWithdraw>(
                client.clone(),
                withdraw_url,
                key,
                signature,
                expiry,
            );

            // Make deposit future
            let deposit_message = format!("{}{}{}", request_path_deposit, &query, expiry);
            let signature = sign_message_phemex(deposit_message, secret);

            let deposit_url = format!(
                "{}{}?{}",
                PHEMEX_BASE_HTTP_URL, request_path_deposit, &query,
            );

            let deposit_future = get_network_info_phemex::<PhemexDeposit>(
                client.clone(),
                deposit_url,
                key,
                signature,
                expiry,
            );

            // Join withdraw and deposit data for given coin
            let (withdraw, deposit) = try_join!(withdraw_future, deposit_future)?;

            // Group withdraw and deposit data
            let mut grouped_data: HashMap<
                String,
                (Option<PhemexWithdrawChainInfo>, Option<PhemexDepositData>),
            > = HashMap::new();

            for withdraw_info in withdraw.data.chain_infos.into_iter() {
                grouped_data
                    .entry(withdraw_info.chain_name.clone())
                    .and_modify(|(withdraw, _)| *withdraw = Some(withdraw_info.clone()))
                    .or_insert((Some(withdraw_info), None));
            }

            for deposit_info in deposit.data.into_iter() {
                grouped_data
                    .entry(deposit_info.chain_name.clone())
                    .and_modify(|(_, deposit)| *deposit = Some(deposit_info.clone()))
                    .or_insert((None, Some(deposit_info)));
            }

            let chain_specs = grouped_data
                .into_iter()
                .map(|(key, (withdraw, deposit))| ChainSpecs::from((key, withdraw, deposit)))
                .collect::<Vec<ChainSpecs>>();

            network_specs.insert(
                (ExchangeId::PhemexSpot, Coin(coin.clone())),
                NetworkSpecData(chain_specs),
            );
        }

        Ok(NetworkSpecs(network_specs))
    }
}

fn sign_message_phemex(message: String, secret: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(message.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

async fn get_network_info_phemex<DeStruct: for<'de> Deserialize<'de>>(
    client: reqwest::Client,
    url: String,
    key: &str,
    signature: String,
    expiry: u64,
) -> Result<DeStruct, SocketError> {
    client
        .get(url)
        .header("x-phemex-access-token", key)
        .header("x-phemex-request-signature", signature)
        .header("x-phemex-request-expiry", expiry.to_string())
        .send()
        .await
        .map_err(SocketError::Http)?
        .json::<DeStruct>()
        .await
        .map_err(SocketError::Http)
}

/*----- */
// Stream selector
/*----- */
impl StreamSelector<PhemexSpotPublicData, OrderBookL2> for PhemexSpotPublicData {
    type Stream = PhemexOrderBookUpdate;
    type StreamTransformer =
        MultiBookTransformer<PhemexSpotPublicData, PhemexSpotBookUpdater, OrderBookL2>;
}

impl StreamSelector<PhemexSpotPublicData, Trades> for PhemexSpotPublicData {
    type Stream = PhemexTradesUpdate;
    type StreamTransformer = StatelessTransformer<PhemexSpotPublicData, Self::Stream, Trades>;
}
