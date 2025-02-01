use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{de, Deserialize, Deserializer};

use crate::{
    assets::level::Level,
    error::SocketError,
    exchange::Identifier,
    model::{
        event_book_snapshot::EventOrderBookSnapshot,
        event_trade::EventTrade,
        market_event::MarketEvent,
        network_info::{ChainSpecs, NetworkSpecData, NetworkSpecs},
    },
    shared::{
        de::{de_str, de_u64_epoch_ms_as_datetime_utc},
        subscription_models::{ExchangeId, Instrument},
    },
    streams::validator::Validator,
};

/*----- */
// OrderBook Snapshot
/*----- */
#[derive(Debug, Default, Deserialize)]
pub struct ExmoOrderBookSnapshot {
    #[serde(deserialize_with = "de_u64_epoch_ms_as_datetime_utc")]
    pub ts: DateTime<Utc>,
    pub event: String,
    pub topic: String,
    pub data: ExmoOrderBookSnapshotData,
}

#[derive(Debug, Deserialize, Default)]
pub struct ExmoOrderBookSnapshotData {
    #[serde(deserialize_with = "de_levels_exmo")]
    pub ask: Vec<Level>,
    #[serde(deserialize_with = "de_levels_exmo")]
    pub bid: Vec<Level>,
}
impl Identifier<String> for ExmoOrderBookSnapshot {
    fn id(&self) -> String {
        self.topic.split(':').last().unwrap_or_default().to_owned()
    }
}

impl From<(ExmoOrderBookSnapshot, Instrument)> for MarketEvent<EventOrderBookSnapshot> {
    fn from((value, instrument): (ExmoOrderBookSnapshot, Instrument)) -> Self {
        Self {
            exchange_time: value.ts,
            received_time: Utc::now(),
            exchange: ExchangeId::ExmoSpot,
            instrument,
            event_data: EventOrderBookSnapshot {
                bids: value.data.bid,
                asks: value.data.ask,
            },
        }
    }
}

fn de_levels_exmo<'de, D>(deserializer: D) -> Result<Vec<Level>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw_data: Vec<[&str; 3]> = Vec::deserialize(deserializer)?;

    Ok(raw_data
        .into_iter()
        .map(|entry| Level {
            price: entry[0].parse().unwrap(),
            size: entry[1].parse().unwrap(),
        })
        .collect())
}

/*----- */
// Trade
/*----- */
#[derive(Debug, Deserialize, Default)]
pub struct ExmoTrades {
    #[serde(deserialize_with = "de_u64_epoch_ms_as_datetime_utc")]
    pub ts: DateTime<Utc>,
    pub event: String,
    pub topic: String,
    pub data: Vec<ExmoTradesData>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ExmoTradesData {
    pub trade_id: u64,
    #[serde(rename = "type", deserialize_with = "de_buyer_is_maker_exmo")]
    pub trade_type: bool,
    #[serde(deserialize_with = "de_str")]
    pub price: f64,
    #[serde(deserialize_with = "de_str")]
    pub quantity: f64,
    #[serde(deserialize_with = "de_str")]
    pub amount: f64,
    pub date: u64,
}

impl Identifier<String> for ExmoTrades {
    fn id(&self) -> String {
        self.topic.split(':').last().unwrap_or_default().to_owned()
    }
}

impl From<(ExmoTrades, Instrument)> for MarketEvent<Vec<EventTrade>> {
    fn from((event, instrument): (ExmoTrades, Instrument)) -> Self {
        Self {
            exchange_time: event.ts,
            received_time: Utc::now(),
            exchange: ExchangeId::ExmoSpot,
            instrument,
            event_data: event
                .data
                .iter()
                .map(|trade_data| {
                    EventTrade::new(
                        Level::new(trade_data.price, trade_data.quantity),
                        trade_data.trade_type,
                    )
                })
                .collect::<Vec<EventTrade>>(),
        }
    }
}

pub fn de_buyer_is_maker_exmo<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    <&str as Deserialize>::deserialize(deserializer).map(|buyer_is_maker| buyer_is_maker == "buy")
}

/*----- */
// Subscription Responses
/*----- */
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ExmoSubscriptionResponse {
    ConnectionSucess {
        ts: u64,
        event: String,
        code: u64,
        message: String,
        session_id: String,
    },
    SubscriptionSuccess {
        ts: u64,
        event: String,
        id: u64,
        topic: String,
    },
    SubscriptionError {
        ts: u64,
        event: String,
        code: u64,
        message: String,
    },
}

impl Validator for ExmoSubscriptionResponse {
    fn validate(self) -> Result<Self, SocketError> {
        match self {
            ExmoSubscriptionResponse::ConnectionSucess { .. } => Ok(self),
            ExmoSubscriptionResponse::SubscriptionSuccess { .. } => Ok(self),
            ExmoSubscriptionResponse::SubscriptionError { message, .. } => {
                Err(SocketError::Subscribe(format!(
                    "received failure subscription response for Exmo. Error msg: {:?}",
                    message
                )))
            }
        }
    }
}

/*----- */
// Network info
/*----- */
#[derive(Debug, Deserialize)]
pub struct ExmoNetworkInfo {
    #[serde(flatten)]
    pub networks: HashMap<String, Vec<ExmoNetworkInfoData>>,
}

#[derive(Debug, Deserialize)]
pub struct ExmoNetworkInfoData {
    #[serde(rename = "type")]
    pub transaction_type: String,
    pub name: String,
    pub currency_name: String,
    pub min: String,
    pub max: String,
    pub enabled: bool,
    pub comment: String,
    #[serde(deserialize_with = "de_withdraw_amount_exmo")]
    pub commission_desc: f64,
    pub currency_confirmations: u32,
}

impl From<ExmoNetworkInfo> for NetworkSpecs {
    fn from(value: ExmoNetworkInfo) -> Self {
        let mut network_specs = Vec::with_capacity(value.networks.len());

        for (coin_name, networks) in value.networks.into_iter() {
            // Exmo sends deposit and withdraw data for a given chain separately, So we
            // have to add to a hashmap first to aggregate it. Then edit the chain specs
            // corresponding to the relevant data.
            let mut paired: HashMap<String, Vec<ExmoNetworkInfoData>> =
                HashMap::with_capacity(networks.len());

            for chain_info in networks.into_iter() {
                paired
                    .entry(chain_info.name.clone())
                    .or_default()
                    .push(chain_info)
            }

            let mut chain_specs = Vec::new();
            for (chain_name, chain_spec) in paired.into_iter() {
                let mut chain_spec_default = ChainSpecs {
                    chain_name,
                    fee_is_fixed: true,
                    fees: 0.0,
                    can_deposit: false,
                    can_withdraw: false
                };

                for chain in chain_spec.into_iter() {
                    if chain.transaction_type == "withdraw" {
                        chain_spec_default.can_withdraw = chain.enabled;
                        chain_spec_default.fees = chain.commission_desc;
                    } else {
                        chain_spec_default.can_deposit = chain.enabled;
                    }
                }

                chain_specs.push(chain_spec_default)
            }

            network_specs.push(NetworkSpecData {
                coin: coin_name,
                exchange: ExchangeId::ExmoSpot,
                chains: chain_specs,
            })
        }

        NetworkSpecs(network_specs)
    }
}

pub fn de_withdraw_amount_exmo<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = serde::de::Deserialize::deserialize(deserializer)?;

    // Deposit example: "0%" or ""
    let trimmed = s.trim();
    if trimmed.is_empty() || trimmed == "0%" {
        return Ok(0.0);
    }

    // Withdraw example: "0.0001 trx"
    let amount = s
        .split_whitespace()
        .next()
        .ok_or_else(|| de::Error::custom("missing amount value"))?;

    amount.parse::<f64>().map_err(de::Error::custom)
}
