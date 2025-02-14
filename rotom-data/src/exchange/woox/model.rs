use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::Deserialize;

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
        de::de_u64_epoch_ms_as_datetime_utc,
        subscription_models::{Coin, ExchangeId, Instrument},
    },
    streams::validator::Validator,
};

/*----- */
// OrderBook Snapshot
/*----- */
#[derive(Debug, Default, Deserialize)]
pub struct WooxOrderBookSnapshot {
    pub topic: String,
    #[serde(deserialize_with = "de_u64_epoch_ms_as_datetime_utc")]
    pub ts: DateTime<Utc>,
    pub data: WooxOrderBookSnapshotData,
}

#[derive(Debug, Default, Deserialize)]
pub struct WooxOrderBookSnapshotData {
    pub symbol: String,
    pub asks: Vec<Level>,
    pub bids: Vec<Level>,
}

impl Identifier<String> for WooxOrderBookSnapshot {
    fn id(&self) -> String {
        self.data.symbol.clone()
    }
}

impl From<(WooxOrderBookSnapshot, Instrument)> for MarketEvent<EventOrderBookSnapshot> {
    fn from((value, instrument): (WooxOrderBookSnapshot, Instrument)) -> Self {
        Self {
            exchange_time: value.ts,
            received_time: Utc::now(),
            exchange: ExchangeId::HtxSpot,
            instrument,
            event_data: EventOrderBookSnapshot {
                bids: value.data.bids,
                asks: value.data.asks,
            },
        }
    }
}

/*----- */
// Trade data
/*----- */
#[derive(Debug, Default, Deserialize)]
pub struct WooxTrade {
    pub topic: String,
    #[serde(deserialize_with = "de_u64_epoch_ms_as_datetime_utc")]
    pub ts: DateTime<Utc>,
    pub data: WooxTradeData,
}

#[derive(Debug, Default, Deserialize)]
pub struct WooxTradeData {
    pub symbol: String,
    pub price: f64,
    pub size: f64,
    #[serde(deserialize_with = "de_buyer_is_maker_woox")]
    pub side: bool,
    pub source: u8,
}

impl Identifier<String> for WooxTrade {
    fn id(&self) -> String {
        self.data.symbol.clone()
    }
}

impl From<(WooxTrade, Instrument)> for MarketEvent<EventTrade> {
    fn from((event, instrument): (WooxTrade, Instrument)) -> Self {
        Self {
            exchange_time: event.ts,
            received_time: Utc::now(),
            exchange: ExchangeId::WooxSpot,
            instrument,
            event_data: EventTrade::new(
                Level::new(event.data.price, event.data.size),
                event.data.side,
            ),
        }
    }
}

fn de_buyer_is_maker_woox<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    <&str as Deserialize>::deserialize(deserializer).map(|buyer_is_maker| buyer_is_maker == "BUY")
}

/*----- */
// Subscription Response
/*----- */
#[derive(Debug, Deserialize, PartialEq)]
pub struct WooxSubscriptionResponse {
    id: String,
    event: String,
    success: bool,
    ts: u64,
    #[serde(default)]
    data: String,
    #[serde(default)]
    #[serde(rename = "errorMsg")]
    error_msg: String,
}

impl Validator for WooxSubscriptionResponse {
    fn validate(self) -> Result<Self, SocketError> {
        if self.success {
            Ok(self)
        } else {
            Err(SocketError::Subscribe(
                "received failure subscription response for WooxSpot".to_owned(),
            ))
        }
    }
}

/*----- */
// Network Info
/*----- */
#[derive(Debug, Deserialize)]
pub struct WooxNetworkInfo {
    pub success: bool,
    pub rows: Vec<WooxNetworkInfoData>,
}

#[derive(Debug, Deserialize)]
pub struct WooxNetworkInfoData {
    pub protocol: String,
    pub network: String,
    pub token: String,
    pub name: String,
    #[serde(rename = "minimum_withdrawal")]
    pub minimum_withdrawal: f64,
    #[serde(rename = "withdrawal_fee")]
    pub withdrawal_fee: f64,
    #[serde(rename = "allow_deposit", deserialize_with = "de_network_info_woox")]
    pub allow_deposit: bool,
    #[serde(rename = "allow_withdraw", deserialize_with = "de_network_info_woox")]
    pub allow_withdraw: bool,
}

impl From<WooxNetworkInfo> for NetworkSpecs {
    fn from(value: WooxNetworkInfo) -> Self {
        let mut grouped_data: HashMap<String, Vec<WooxNetworkInfoData>> = HashMap::new();
        for coin in value.rows.into_iter() {
            grouped_data
                .entry(coin.token.clone())
                .or_default()
                .push(coin);
        }

        let network_spec_data = grouped_data
            .into_iter()
            .map(|(coin_name, coin_info)| {
                let chain_specs = coin_info
                    .into_iter()
                    .map(|chain| ChainSpecs {
                        chain_name: chain.protocol.clone(),
                        fee_is_fixed: false,
                        fees: chain.withdrawal_fee,
                        can_deposit: chain.allow_deposit,
                        can_withdraw: chain.allow_withdraw,
                    })
                    .collect::<Vec<ChainSpecs>>();

                (
                    (ExchangeId::WooxSpot, Coin(coin_name)),
                    NetworkSpecData(chain_specs),
                )
            })
            .collect::<HashMap<_, _>>();

        NetworkSpecs(network_spec_data)
    }
}

pub fn de_network_info_woox<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    <u32 as Deserialize>::deserialize(deserializer)
        .map(|deposit_withdraw_status| deposit_withdraw_status == 1)
}
