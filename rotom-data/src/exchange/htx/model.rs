use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer};

use crate::model::event_trade::EventTrade;
use crate::model::network_info::{ChainSpecs, NetworkSpecData, NetworkSpecs};
use crate::shared::de::{de_str_optional, de_u64_epoch_ms_as_datetime_utc, de_uppercase};
use crate::{
    assets::level::Level,
    error::SocketError,
    exchange::Identifier,
    model::{event_book_snapshot::EventOrderBookSnapshot, market_event::MarketEvent},
    shared::subscription_models::{ExchangeId, Instrument},
    streams::validator::Validator,
};

/*----- */
// Orderbook snapshot
/*----- */
#[derive(Debug, Default, Deserialize)]
pub struct HtxOrderBookSnapshot {
    pub ch: String,
    #[serde(deserialize_with = "de_u64_epoch_ms_as_datetime_utc")]
    pub ts: DateTime<Utc>,
    pub tick: HtxOrderBookSnapshotTick,
}

#[derive(Debug, Default, Deserialize)]
pub struct HtxOrderBookSnapshotTick {
    #[serde(rename = "seqNum")]
    pub seq_num: i64,
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
}

// todo: change from string to struct()
// have to split as ws data comes in like "market.glmrusdt.mbp.refresh.5" and we cant have this in stateless transformer
impl Identifier<String> for HtxOrderBookSnapshot {
    fn id(&self) -> String {
        self.ch.split('.').nth(1).unwrap_or_default().to_owned()
    }
}

impl From<(HtxOrderBookSnapshot, Instrument)> for MarketEvent<EventOrderBookSnapshot> {
    fn from((value, instrument): (HtxOrderBookSnapshot, Instrument)) -> Self {
        Self {
            exchange_time: value.ts,
            received_time: Utc::now(),
            exchange: ExchangeId::HtxSpot,
            instrument,
            event_data: EventOrderBookSnapshot {
                bids: value.tick.bids,
                asks: value.tick.asks,
            },
        }
    }
}

/*----- */
// Trade data
/*----- */
#[derive(Debug, Default, Deserialize)]
pub struct HtxTrade {
    pub ch: String,
    pub ts: u64,
    pub tick: HtxTradeData,
}

#[derive(Debug, Default, Deserialize)]
pub struct HtxTradeData {
    pub id: u64,
    #[serde(deserialize_with = "de_u64_epoch_ms_as_datetime_utc")]
    pub ts: DateTime<Utc>,
    pub data: Vec<HtxTradeTick>,
}

#[derive(Debug, Default, Deserialize)]
pub struct HtxTradeTick {
    pub id: u128,
    pub ts: u64,
    #[serde(rename = "tradeId")]
    pub trade_id: u64,
    pub amount: f64,
    pub price: f64,
    #[serde(deserialize_with = "de_buyer_is_maker_htx")]
    pub direction: bool,
}

impl Identifier<String> for HtxTrade {
    fn id(&self) -> String {
        self.ch.split('.').nth(1).unwrap_or_default().to_owned()
    }
}

impl From<(HtxTrade, Instrument)> for MarketEvent<Vec<EventTrade>> {
    fn from((event, instrument): (HtxTrade, Instrument)) -> Self {
        Self {
            exchange_time: event.tick.ts,
            received_time: Utc::now(),
            exchange: ExchangeId::HtxSpot,
            instrument,
            event_data: event
                .tick
                .data
                .iter()
                .map(|trade_data| {
                    EventTrade::new(
                        Level::new(trade_data.price, trade_data.amount),
                        trade_data.direction,
                    )
                })
                .collect::<Vec<EventTrade>>(),
        }
    }
}

pub fn de_buyer_is_maker_htx<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    <&str as Deserialize>::deserialize(deserializer).map(|buyer_is_maker| buyer_is_maker == "buy")
}

/*----- */
// Subscription response
/*----- */
#[derive(Debug, Deserialize, PartialEq)]
pub struct HtxSubscriptionResponse {
    #[serde(skip)]
    id: u128,
    status: String,
    #[serde(default)]
    subbed: String,
    ts: u64,
    #[serde(rename = "err-code", default)]
    err_code: String,
    #[serde(rename = "err-msg", default)]
    err_msg: String,
}

impl Validator for HtxSubscriptionResponse {
    fn validate(self) -> Result<Self, SocketError> {
        if self.status == *"ok" {
            Ok(self)
        } else {
            Err(SocketError::Subscribe(format!(
                "Error while subscribing to htx spot, error: {:#?}",
                self.err_msg
            )))
        }
    }
}

/*----- */
// Wallet Info
/*----- */
// Ref: https://www.htx.com/en-us/opend/newApiPages/?id=7ec516fc-7773-11ed-9966-0242ac110003
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HtxNetworkInfo {
    pub code: u32,
    pub data: Vec<HtxNetworkCurrencyInfo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HtxNetworkCurrencyInfo {
    pub asset_type: i32,
    pub chains: Vec<HtxChainInfo>,
    #[serde(deserialize_with = "de_uppercase")]
    pub currency: String,
    pub inst_status: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HtxChainInfo {
    pub addr_deposit_tag: bool,
    pub addr_with_tag: bool,
    pub base_chain: Option<String>,
    pub base_chain_protocol: Option<String>,
    pub chain: String,
    #[serde(deserialize_with = "de_withdraw_deposit_status_htx")]
    pub deposit_status: bool,
    pub display_name: String,
    pub full_name: String,
    pub is_dynamic: Option<bool>,
    pub max_withdraw_amt: String,
    pub min_deposit_amt: String,
    pub min_withdraw_amt: String,
    pub num_of_confirmations: i32,
    pub num_of_fast_confirmations: i32,
    #[serde(deserialize_with = "de_str_optional", default)]
    pub transact_fee_withdraw: Option<f64>,
    #[serde(deserialize_with = "de_str_optional", default)]
    pub transact_fee_rate_withdraw: Option<f64>,
    pub withdraw_fee_type: String,
    pub withdraw_precision: i32,
    pub withdraw_quota_per_day: String,
    pub withdraw_quota_per_year: Option<String>,
    pub withdraw_quota_total: Option<String>,
    #[serde(deserialize_with = "de_withdraw_deposit_status_htx")]
    pub withdraw_status: bool,
}

impl From<HtxNetworkInfo> for NetworkSpecs {
    fn from(value: HtxNetworkInfo) -> Self {
        let network_spec_data = value
            .data
            .iter()
            .map(|coin| {
                let chain_specs = coin
                    .chains
                    .iter()
                    .map(|chain|  {
                        // Fill in chain spec info as much as possible
                        let mut chain_spec = ChainSpecs {
                            chain_name: chain.display_name.clone(),
                            fee_is_fixed: true,
                            fees: 0.0,
                            can_deposit: chain.deposit_status,
                            can_withdraw: chain.withdraw_status,
                        };

                        // Then based on if the fee type is fixed or not, fill in the fields
                        if chain.withdraw_fee_type == "fixed" {
                            chain_spec.fees = chain.transact_fee_withdraw
                                .expect("Fee type is fixed but got a none value for transact_fee_withdraw, this should never happen"); // should never fail
                        } else {
                            chain_spec.fees = chain.transact_fee_rate_withdraw
                                .expect("Fee type is fixed but got a none value for transact_fee_rate_withdraw, this should never happen"); // should never fail 
                            chain_spec.fee_is_fixed = false;
                        }

                        chain_spec
                    })
                    .collect::<Vec<ChainSpecs>>();

                NetworkSpecData {
                    coin: coin.currency.clone(),
                    exchange: ExchangeId::HtxSpot,
                    chains: chain_specs,
                }
            })
            .collect::<Vec<NetworkSpecData>>();

        NetworkSpecs(network_spec_data)
    }
}

fn de_withdraw_deposit_status_htx<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    <&str as Deserialize>::deserialize(deserializer)
        .map(|withdraw_status| withdraw_status == "allowed")
}
