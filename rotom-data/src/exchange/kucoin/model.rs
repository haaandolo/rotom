use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::assets::level::Level;
use crate::error::SocketError;
use crate::exchange::Identifier;
use crate::model::event_book_snapshot::EventOrderBookSnapshot;
use crate::model::event_trade::EventTrade;
use crate::model::market_event::MarketEvent;
use crate::model::network_info::{ChainSpecs, NetworkSpecData, NetworkSpecs};
use crate::shared::de::{
    de_str, de_str_optional, de_str_u64_epoch_ns_as_datetime_utc, de_u64_epoch_ms_as_datetime_utc,
};
use crate::shared::subscription_models::{ExchangeId, Instrument};
use crate::streams::validator::Validator;

/*----- */
// Kucoin Ws URL response
/*----- */
#[derive(Debug, Deserialize)]
pub struct KuCoinWsUrl {
    pub code: String,
    pub data: KuCoinWsUrlData,
}

#[derive(Debug, Deserialize)]
pub struct KuCoinWsUrlData {
    pub token: String,
    #[serde(rename = "instanceServers")]
    pub instance_servers: Vec<KuCoinInstanceServer>,
}

#[derive(Debug, Deserialize)]
pub struct KuCoinInstanceServer {
    pub endpoint: String,
    pub encrypt: bool,
    pub protocol: String,
    #[serde(rename = "pingInterval")]
    pub ping_interval: u32,
    #[serde(rename = "pingTimeout")]
    pub ping_timeout: u32,
}

/*----- */
// OrderBook Snapshot
/*----- */
#[derive(Debug, Deserialize, Default)]
pub struct KuCoinOrderBookSnapshot {
    pub topic: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub subject: String,
    pub data: KuCoinOrderBookSnapshotData,
}

#[derive(Debug, Deserialize, Default)]
pub struct KuCoinOrderBookSnapshotData {
    pub asks: Vec<Level>,
    pub bids: Vec<Level>,
    #[serde(deserialize_with = "de_u64_epoch_ms_as_datetime_utc")]
    pub timestamp: DateTime<Utc>,
}

impl Identifier<String> for KuCoinOrderBookSnapshot {
    fn id(&self) -> String {
        self.topic.split(':').last().unwrap_or_default().to_owned()
    }
}

impl From<(KuCoinOrderBookSnapshot, Instrument)> for MarketEvent<EventOrderBookSnapshot> {
    fn from((value, instrument): (KuCoinOrderBookSnapshot, Instrument)) -> Self {
        Self {
            exchange_time: value.data.timestamp,
            received_time: Utc::now(),
            exchange: ExchangeId::KuCoinSpot,
            instrument,
            event_data: EventOrderBookSnapshot {
                bids: value.data.bids,
                asks: value.data.asks,
            },
        }
    }
}

/*----- */
// Trade
/*----- */
#[derive(Debug, Deserialize, Default)]
pub struct KuCoinTrade {
    pub topic: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub subject: String,
    pub data: KuCoinTradeData,
}

#[derive(Debug, Deserialize, Default)]
pub struct KuCoinTradeData {
    #[serde(rename = "makerOrderId")]
    pub maker_order_id: String,
    #[serde(deserialize_with = "de_str")]
    pub price: f64,
    pub sequence: String,
    #[serde(deserialize_with = "de_buyer_is_maker_kucoin")]
    pub side: bool,
    #[serde(deserialize_with = "de_str")]
    pub size: f64,
    pub symbol: String,
    #[serde(rename = "takerOrderId")]
    pub taker_order_id: String,
    #[serde(deserialize_with = "de_str_u64_epoch_ns_as_datetime_utc")]
    pub time: DateTime<Utc>,
    #[serde(rename = "tradeId")]
    pub trade_id: String,
    #[serde(rename = "type")]
    pub trade_type: String,
}

impl Identifier<String> for KuCoinTrade {
    fn id(&self) -> String {
        self.data.symbol.clone()
    }
}

impl From<(KuCoinTrade, Instrument)> for MarketEvent<EventTrade> {
    fn from((event, instrument): (KuCoinTrade, Instrument)) -> Self {
        Self {
            exchange_time: event.data.time,
            received_time: Utc::now(),
            exchange: ExchangeId::KuCoinSpot,
            instrument,
            event_data: EventTrade::new(
                Level::new(event.data.price, event.data.size),
                event.data.side,
            ),
        }
    }
}

fn de_buyer_is_maker_kucoin<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    <&str as Deserialize>::deserialize(deserializer).map(|buyer_is_maker| buyer_is_maker == "buy")
}

/*----- */
// Subscription Response
/*----- */
#[derive(Debug, Deserialize)]
pub struct KuCoinSubscriptionResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub response_type: String,
    #[serde(default)]
    pub code: u64,
    #[serde(default)]
    pub data: String,
}

impl Validator for KuCoinSubscriptionResponse {
    fn validate(self) -> Result<Self, SocketError> {
        if self.response_type == *"error" {
            Err(SocketError::Subscribe(
                format!("received failure subscription response for kucoin. Error code: {}. Error message {}", self.code, self.data),
            ))
        } else {
            Ok(self)
        }
    }
}

/*----- */
// Network information
/*----- */
#[derive(Debug, Deserialize)]
pub struct KuCoinNetworkInfo {
    #[serde(rename = "code")]
    pub code: String,
    #[serde(rename = "data")]
    pub data: Vec<KuCoinNetworkInfoData>,
}

#[derive(Debug, Deserialize)]
pub struct KuCoinNetworkInfoData {
    pub currency: String,
    pub name: String,
    #[serde(rename = "fullName")]
    pub full_name: String,
    pub precision: u8,
    pub confirms: Option<u8>,
    #[serde(rename = "contractAddress")]
    pub contract_address: Option<String>,
    #[serde(rename = "isMarginEnabled")]
    pub is_margin_enabled: bool,
    #[serde(rename = "isDebitEnabled")]
    pub is_debit_enabled: bool,
    pub chains: Option<Vec<KuCoinNetworkChain>>,
}

#[derive(Debug, Deserialize)]
pub struct KuCoinNetworkChain {
    #[serde(rename = "chainName")]
    pub chain_name: String,
    #[serde(rename = "withdrawalMinSize")]
    pub withdrawal_min_size: String,
    #[serde(rename = "depositMinSize")]
    pub deposit_min_size: Option<String>,
    #[serde(
        rename = "withdrawFeeRate",
        deserialize_with = "de_str_optional",
        default
    )]
    pub withdraw_fee_rate: Option<f64>,
    #[serde(rename = "withdrawalMinFee", deserialize_with = "de_str")]
    pub withdrawal_min_fee: f64,
    #[serde(rename = "isWithdrawEnabled")]
    pub is_withdraw_enabled: bool,
    #[serde(rename = "isDepositEnabled")]
    pub is_deposit_enabled: bool,
    pub confirms: u32,
    #[serde(rename = "preConfirms")]
    pub pre_confirms: u32,
    #[serde(rename = "contractAddress")]
    pub contract_address: String,
    #[serde(rename = "withdrawPrecision")]
    pub withdraw_precision: u8,
    #[serde(rename = "maxWithdraw")]
    pub max_withdraw: Option<String>,
    #[serde(rename = "maxDeposit")]
    pub max_deposit: Option<String>,
    #[serde(rename = "needTag")]
    pub need_tag: bool,
    #[serde(rename = "chainId")]
    pub chain_id: String,
}

impl From<KuCoinNetworkInfo> for NetworkSpecs {
    fn from(value: KuCoinNetworkInfo) -> Self {
        let network_spec_data = value
            .data
            .into_iter()
            .map(|coin| {
                let chain_specs = coin
                    .chains
                    .unwrap_or_default()
                    .into_iter()
                    .map(|chain| {
                        let mut chain_spec = ChainSpecs {
                            chain_name: chain.chain_name.clone(),
                            fee_is_fixed: true,
                            fees: chain.withdrawal_min_fee,
                            can_deposit: chain.is_deposit_enabled,
                            can_withdraw: chain.is_withdraw_enabled,
                        };

                        // If fee rate is not None change chain spec to reflect this
                        if let Some(rate) = chain.withdraw_fee_rate {
                            chain_spec.fees = rate;
                            chain_spec.fee_is_fixed = false;
                        }

                        chain_spec
                    })
                    .collect::<Vec<ChainSpecs>>();

                NetworkSpecData {
                    coin: coin.currency,
                    exchange: ExchangeId::KuCoinSpot,
                    chains: chain_specs,
                }
            })
            .collect::<Vec<NetworkSpecData>>();

        NetworkSpecs(network_spec_data)
    }
}
