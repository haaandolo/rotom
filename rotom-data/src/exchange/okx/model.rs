use std::mem;

use crate::{
    error::SocketError,
    exchange::Identifier,
    model::{
        event_book_snapshot::EventOrderBookSnapshot, event_trade::EventTrade,
        market_event::MarketEvent,
    },
    shared::{
        de::{de_str, de_str_u64_epoch_ms_as_datetime_utc},
        subscription_models::{ExchangeId, Instrument},
    },
    streams::validator::Validator,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::assets::level::Level;

/*----- */
// OrderBook snapshot
/*----- */
#[derive(Debug, Default, Deserialize)]
pub struct OkxOrderBookSnapshot {
    pub arg: OkxOrderBookSnapshotArg,
    pub data: [OkxOrderBookSnapshotData; 1],
}

#[derive(Debug, Default, Deserialize)]
pub struct OkxOrderBookSnapshotArg {
    pub channel: String,
    #[serde(rename = "instId")]
    pub inst_id: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct OkxOrderBookSnapshotData {
    #[serde(deserialize_with = "de_levels_okx")]
    pub asks: Vec<Level>,
    #[serde(deserialize_with = "de_levels_okx")]
    pub bids: Vec<Level>,
    #[serde(rename = "instId")]
    pub inst_id: String,
    #[serde(deserialize_with = "de_str_u64_epoch_ms_as_datetime_utc")]
    pub ts: DateTime<Utc>,
    #[serde(rename = "seqId")]
    pub seq_id: u64,
}

impl Identifier<String> for OkxOrderBookSnapshot {
    fn id(&self) -> String {
        self.arg.channel.clone()
    }
}

impl From<(OkxOrderBookSnapshot, Instrument)> for MarketEvent<EventOrderBookSnapshot> {
    fn from((mut value, instrument): (OkxOrderBookSnapshot, Instrument)) -> Self {
        let data = mem::take(&mut value.data[0]);
        Self {
            exchange_time: data.ts,
            received_time: Utc::now(),
            exchange: ExchangeId::OkxSpot,
            instrument,
            event_data: EventOrderBookSnapshot {
                bids: data.bids,
                asks: data.asks,
            },
        }
    }
}

fn de_levels_okx<'de, D>(deserializer: D) -> Result<Vec<Level>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw_data: Vec<[&str; 4]> = Vec::deserialize(deserializer)?;

    Ok(raw_data
        .into_iter()
        .map(|entry| Level {
            price: entry[0].parse().unwrap(),
            size: entry[1].parse().unwrap(),
        })
        .collect())
}

/*----- */
// Subscription Response
/*----- */
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum OkxSubscriptionResponse {
    Success {
        event: String,
        arg: serde_json::Value,
        #[serde(rename = "connId")]
        conn_id: String,
    },
    Error {
        event: String,
        msg: String,
        code: String,
        #[serde(rename = "connId")]
        conn_id: String,
    },
}

impl Validator for OkxSubscriptionResponse {
    fn validate(self) -> Result<Self, SocketError> {
        match self {
            OkxSubscriptionResponse::Success { .. } => Ok(self),
            OkxSubscriptionResponse::Error { msg, .. } => Err(SocketError::Subscribe(format!(
                "received failure subscription response for Okx. Error msg: {:?}",
                msg
            ))),
        }
    }
}

/*----- */
// Okx Trades
/*----- */
#[derive(Debug, Default, Deserialize)]
pub struct OkxTrade {
    pub arg: OkxTradeArg,
    pub data: [OkxTradeData; 1],
}

#[derive(Debug, Default, Deserialize)]
pub struct OkxTradeArg {
    pub channel: String,
    #[serde(rename = "instId")]
    pub inst_id: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct OkxTradeData {
    #[serde(rename = "instId")]
    pub inst_id: String,
    #[serde(rename = "tradeId")]
    pub trade_id: String,
    #[serde(deserialize_with = "de_str")]
    pub px: f64,
    #[serde(deserialize_with = "de_str")]
    pub sz: f64,
    #[serde(deserialize_with = "de_buyer_is_maker_okx")]
    pub side: bool,
    #[serde(deserialize_with = "de_str_u64_epoch_ms_as_datetime_utc")]
    pub ts: DateTime<Utc>,
    pub count: String,
}

impl Identifier<String> for OkxTrade {
    fn id(&self) -> String {
        self.arg.inst_id.clone()
    }
}

impl From<(OkxTrade, Instrument)> for MarketEvent<EventTrade> {
    fn from((event, instrument): (OkxTrade, Instrument)) -> Self {
        Self {
            exchange_time: event.data[0].ts,
            received_time: Utc::now(),
            exchange: ExchangeId::OkxSpot,
            instrument,
            event_data: EventTrade::new(
                Level::new(event.data[0].px, event.data[0].sz),
                event.data[0].side,
            ),
        }
    }
}

pub fn de_buyer_is_maker_okx<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    <&str as Deserialize>::deserialize(deserializer).map(|buyer_is_maker| buyer_is_maker == "buy")
}

/*----- */
// Network infomation
/*----- */
#[derive(Debug, Deserialize)]
pub struct OkxNetworkInfo {
    #[serde(default)]
    pub msg: String,
    pub data: Vec<OkxNetworkInfoData>,
}

#[derive(Debug, Deserialize)]
pub struct OkxNetworkInfoData {
    #[serde(rename = "burningFeeRate")]
    pub burning_fee_rate: String,
    #[serde(rename = "canDep")]
    pub can_deposit: bool,
    #[serde(rename = "canInternal")]
    pub can_internal: bool,
    #[serde(rename = "canWd")]
    pub can_withdraw: bool,
    pub ccy: String,
    pub chain: String,
    #[serde(rename = "ctAddr")]
    pub contract_address: String,
    #[serde(rename = "depEstOpenTime")]
    pub deposit_estimated_open_time: String,
    #[serde(rename = "depQuotaFixed")]
    pub deposit_quota_fixed: String,
    #[serde(rename = "depQuoteDailyLayer2")]
    pub deposit_quota_daily_layer2: String,
    #[serde(deserialize_with = "de_str")]
    pub fee: f64,
    #[serde(rename = "logoLink")]
    pub logo_link: String,
    #[serde(rename = "mainNet")]
    pub main_net: bool,
    #[serde(rename = "maxFee")]
    pub max_fee: String,
    #[serde(rename = "maxFeeForCtAddr")]
    pub max_fee_for_contract_address: String,
    #[serde(rename = "maxWd")]
    pub max_withdraw: String,
    #[serde(rename = "minDep")]
    pub min_deposit: String,
    #[serde(rename = "minDepArrivalConfirm")]
    pub min_deposit_arrival_confirm: String,
    #[serde(rename = "minFee")]
    pub min_fee: String,
    #[serde(rename = "minFeeForCtAddr")]
    pub min_fee_for_contract_address: String,
    #[serde(rename = "minInternal")]
    pub min_internal: String,
    #[serde(rename = "minWd")]
    pub min_withdraw: String,
    #[serde(rename = "minWdUnlockConfirm")]
    pub min_withdraw_unlock_confirm: String,
    pub name: String,
    #[serde(rename = "needTag")]
    pub need_tag: bool,
    #[serde(rename = "usedDepQuotaFixed")]
    pub used_deposit_quota_fixed: String,
    #[serde(rename = "usedWdQuota")]
    pub used_withdraw_quota: String,
    #[serde(rename = "wdEstOpenTime")]
    pub withdraw_estimated_open_time: String,
    #[serde(rename = "wdQuota")]
    pub withdraw_quota: String,
    #[serde(rename = "wdTickSz")]
    pub withdraw_tick_size: String,
}
