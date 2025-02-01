use ::serde::Deserialize;
use chrono::{DateTime, Utc};

use crate::assets::level::Level;
use crate::error::SocketError;
use crate::exchange::Identifier;
use crate::model::event_book_snapshot::EventOrderBookSnapshot;
use crate::model::event_trade::EventTrade;
use crate::model::market_event::MarketEvent;
use crate::model::network_info::{ChainSpecs, NetworkSpecData, NetworkSpecs};
use crate::shared::de::{de_str, de_u64_epoch_ms_as_datetime_utc};
use crate::shared::subscription_models::{ExchangeId, Instrument};
use crate::streams::validator::Validator;

/*----- */
// OrderBook Snapshot
/*----- */
#[derive(Debug, Default, Deserialize)]
pub struct CoinExOrderBookSnapshot {
    pub data: CoinExOrderBookSnapshotData,
    pub id: Option<serde_json::Value>,
    pub method: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct CoinExOrderBookSnapshotData {
    pub depth: CoinExOrderBookSnapshotTick,
    #[serde(rename = "is_full")]
    pub is_full: bool,
    pub market: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct CoinExOrderBookSnapshotTick {
    pub asks: Vec<Level>,
    pub bids: Vec<Level>,
    pub checksum: i64,
    pub last: String,
    #[serde(
        rename = "updated_at",
        deserialize_with = "de_u64_epoch_ms_as_datetime_utc"
    )]
    pub updated_at: DateTime<Utc>,
}

impl Identifier<String> for CoinExOrderBookSnapshot {
    fn id(&self) -> String {
        self.data.market.clone()
    }
}

impl From<(CoinExOrderBookSnapshot, Instrument)> for MarketEvent<EventOrderBookSnapshot> {
    fn from((value, instrument): (CoinExOrderBookSnapshot, Instrument)) -> Self {
        Self {
            exchange_time: value.data.depth.updated_at,
            received_time: Utc::now(),
            exchange: ExchangeId::CoinExSpot,
            instrument,
            event_data: EventOrderBookSnapshot {
                bids: value.data.depth.bids,
                asks: value.data.depth.asks,
            },
        }
    }
}

/*----- */
// Trades
/*----- */
#[derive(Debug, Deserialize, Default)]
pub struct CoinExTrade {
    pub method: String,
    pub data: CoinExTradeData,
    pub id: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Default)]
pub struct CoinExTradeData {
    pub market: String,
    pub deal_list: Vec<CoinExTradeTick>,
}

#[derive(Debug, Deserialize, Default)]
pub struct CoinExTradeTick {
    pub deal_id: u64,
    #[serde(deserialize_with = "de_u64_epoch_ms_as_datetime_utc")]
    pub created_at: DateTime<Utc>,
    #[serde(deserialize_with = "de_buyer_is_maker_coinex")]
    pub side: bool,
    #[serde(deserialize_with = "de_str")]
    pub price: f64,
    #[serde(deserialize_with = "de_str")]
    pub amount: f64,
}

impl Identifier<String> for CoinExTrade {
    fn id(&self) -> String {
        self.data.market.clone()
    }
}

impl From<(CoinExTrade, Instrument)> for MarketEvent<Vec<EventTrade>> {
    fn from((event, instrument): (CoinExTrade, Instrument)) -> Self {
        Self {
            exchange_time: event.data.deal_list[0].created_at, // todo: change Vec tradees to have date in each event_data field
            received_time: Utc::now(),
            exchange: ExchangeId::CoinExSpot,
            instrument,
            event_data: event
                .data
                .deal_list
                .iter()
                .map(|trade| EventTrade::new(Level::new(trade.price, trade.amount), trade.side))
                .collect::<Vec<_>>(),
        }
    }
}

pub fn de_buyer_is_maker_coinex<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    <&str as Deserialize>::deserialize(deserializer).map(|buyer_is_maker| buyer_is_maker == "buy")
}

/*----- */
// Subscription Response
/*----- */
#[derive(Debug, Deserialize)]
pub struct CoinExSubscriptionResponse {
    pub id: u64,
    pub code: u64,
    pub message: String,
}

impl Validator for CoinExSubscriptionResponse {
    fn validate(self) -> Result<Self, SocketError> {
        if self.message == *"OK" {
            Ok(self)
        } else {
            Err(SocketError::Subscribe(format!(
                "received failure subscription response for CoinExSpot. Expected OK but got {}",
                self.message
            )))
        }
    }
}

/*----- */
// Network infomation
/*----- */
#[derive(Debug, Deserialize)]
pub struct CoinExNetworkInfo {
    pub code: u32,
    pub data: Vec<CoinExNetworkTicker>,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct CoinExNetworkTicker {
    pub asset: CoinExNetworkTickerInfo,
    pub chains: Vec<CoinExNetworksAvailable>,
}

#[derive(Debug, Deserialize)]
pub struct CoinExNetworkTickerInfo {
    pub ccy: String,
    pub deposit_enabled: bool,
    pub withdraw_enabled: bool,
    pub inter_transfer_enabled: bool,
    pub is_st: bool,
}

#[derive(Debug, Deserialize)]
pub struct CoinExNetworksAvailable {
    pub chain: String,
    pub min_deposit_amount: String,
    pub min_withdraw_amount: String,
    pub deposit_enabled: bool,
    pub withdraw_enabled: bool,
    pub deposit_delay_minutes: i32,
    pub safe_confirmations: i32,
    pub irreversible_confirmations: i32,
    pub deflation_rate: String,
    #[serde(deserialize_with = "de_str")]
    pub withdrawal_fee: f64,
    pub withdrawal_precision: i32,
    pub memo: String,
    pub is_memo_required_for_deposit: bool,
    pub explorer_asset_url: String,
}

impl From<CoinExNetworkInfo> for NetworkSpecs {
    fn from(value: CoinExNetworkInfo) -> Self {
        let network_spec_data = value
            .data
            .iter()
            .map(|coin| {
                let chain_specs = coin
                    .chains
                    .iter()
                    .map(|chain| ChainSpecs {
                        chain_name: chain.chain.clone(),
                        fee_is_fixed: true,
                        fees: chain.withdrawal_fee,
                        can_deposit: chain.deposit_enabled,
                        can_withdraw: chain.withdraw_enabled,
                    })
                    .collect();

                NetworkSpecData {
                    coin: coin.asset.ccy.clone(),
                    exchange: ExchangeId::CoinExSpot,
                    chains: chain_specs,
                }
            })
            .collect::<Vec<_>>();

        NetworkSpecs(network_spec_data)
    }
}
