use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::assets::level::Level;
use crate::error::SocketError;
use crate::exchange::Identifier;
use crate::model::event_trade::EventTrade;
use crate::model::market_event::MarketEvent;
use crate::model::network_info::{ChainSpecs, NetworkSpecs};
use crate::model::ticker_info::TickerInfo;
use crate::shared::de::{de_str, de_u64_epoch_ms_as_datetime_utc};
use crate::shared::subscription_models::{ExchangeId, Instrument};
use crate::streams::validator::Validator;

/*----- */
// OrderBook Update L2
/*----- */
#[derive(Debug, Deserialize)]
pub struct AscendExBookUpdate {
    pub m: String,
    pub symbol: String,
    pub data: AscendExBookUpdateData,
}

#[derive(Debug, Deserialize)]
pub struct AscendExBookUpdateData {
    #[serde(deserialize_with = "de_u64_epoch_ms_as_datetime_utc")]
    pub ts: DateTime<Utc>,
    pub seqnum: u64,
    pub asks: Vec<Level>,
    pub bids: Vec<Level>,
}

impl Identifier<String> for AscendExBookUpdate {
    fn id(&self) -> String {
        self.symbol.clone()
    }
}

/*----- */
// OrderBook Snapshot
/*----- */
#[derive(Debug, Deserialize)]
pub struct AscendExOrderBookSnapshot {
    pub code: u64,
    pub data: AscendExOrderBookSnapshotData,
}

#[derive(Debug, Deserialize)]
pub struct AscendExOrderBookSnapshotData {
    pub m: String,
    pub symbol: String,
    pub data: AscendExOrderBookSnapshotTicks,
}

#[derive(Debug, Deserialize)]
pub struct AscendExOrderBookSnapshotTicks {
    #[serde(deserialize_with = "de_u64_epoch_ms_as_datetime_utc")]
    pub ts: DateTime<Utc>,
    pub seqnum: u64,
    pub asks: Vec<Level>,
    pub bids: Vec<Level>,
}

/*----- */
// Trade
/*----- */
#[derive(Debug, Default, Deserialize)]
pub struct AscendExTrades {
    pub m: String,
    pub symbol: String,
    pub data: Vec<AscendExTradesData>,
}

#[derive(Debug, Default, Deserialize)]
pub struct AscendExTradesData {
    #[serde(deserialize_with = "de_str")]
    pub p: f64,
    #[serde(deserialize_with = "de_str")]
    pub q: f64,
    #[serde(deserialize_with = "de_u64_epoch_ms_as_datetime_utc")]
    pub ts: DateTime<Utc>,
    pub bm: bool,
    pub seqnum: u64,
}

impl Identifier<String> for AscendExTrades {
    fn id(&self) -> String {
        self.symbol.clone()
    }
}

impl From<(AscendExTrades, Instrument)> for MarketEvent<Vec<EventTrade>> {
    fn from((event, instrument): (AscendExTrades, Instrument)) -> Self {
        Self {
            exchange_time: event.data[0].ts, // ts for all trades in the Vex should be the same, so this is allowed
            received_time: Utc::now(),
            exchange: ExchangeId::AscendExSpot,
            instrument,
            event_data: event
                .data
                .iter()
                .map(|trade_data| {
                    EventTrade::new(Level::new(trade_data.p, trade_data.q), trade_data.bm)
                })
                .collect::<Vec<EventTrade>>(),
        }
    }
}

/*----- */
// Subscription Response
/*----- */
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum AscendExSubscriptionResponse {
    ConnectionSuccess {
        m: String,
        #[serde(rename = "type")]
        connection_type: String,
    },
    SubscriptionSuccess {
        m: String,
        id: String,
        ch: String,
        code: u64,
    },
    SubscriptionError {
        m: String,
        id: String,
        reason: String,
        code: u64,
        info: String,
    },
}

impl Validator for AscendExSubscriptionResponse {
    fn validate(self) -> Result<Self, SocketError> {
        match self {
            AscendExSubscriptionResponse::ConnectionSuccess { .. } => Ok(self),
            AscendExSubscriptionResponse::SubscriptionSuccess { .. } => Ok(self),
            AscendExSubscriptionResponse::SubscriptionError { reason, info, .. } => {
                Err(SocketError::Subscribe(format!(
                    "received failure subscription response AscendExSpot. Reason: {}. Info {}",
                    reason, info
                )))
            }
        }
    }
}

/*----- */
// Network info
/*----- */
#[derive(Debug, Deserialize)]
pub struct AscendExNetworkInfo {
    pub code: u64,
    pub data: Vec<AscendExNetworkInfoData>,
}

#[derive(Debug, Deserialize)]
pub struct AscendExNetworkInfoData {
    #[serde(rename = "assetCode")]
    pub asset_code: String,
    #[serde(rename = "assetName")]
    pub asset_name: String,
    #[serde(rename = "precisionScale")]
    pub precision_scale: i32,
    #[serde(rename = "nativeScale")]
    pub native_scale: i32,
    #[serde(rename = "blockChain")]
    pub block_chain: Vec<AscendExNetworkConfig>,
}

#[derive(Debug, Deserialize)]
pub struct AscendExNetworkConfig {
    #[serde(rename = "chainName")]
    pub chain_name: String,
    #[serde(rename = "withdrawFee", deserialize_with = "de_str")]
    pub withdraw_fee: f64,
    #[serde(rename = "allowDeposit")]
    pub allow_deposit: bool,
    #[serde(rename = "allowWithdraw")]
    pub allow_withdraw: bool,
    #[serde(rename = "minDepositAmt")]
    pub min_deposit_amt: String,
    #[serde(rename = "minWithdrawal")]
    pub min_withdrawal: String,
    #[serde(rename = "numConfirmations")]
    pub num_confirmations: i32,
}

impl From<AscendExNetworkInfo> for Vec<NetworkSpecs> {
    fn from(value: AscendExNetworkInfo) -> Self {
        value
            .data
            .iter()
            .map(|coin| {
                let chain_specs = coin
                    .block_chain
                    .iter()
                    .map(|chain| ChainSpecs {
                        chain_name: chain.chain_name.clone(),
                        fees: chain.withdraw_fee,
                        can_deposit: chain.allow_deposit,
                        can_withdraw: chain.allow_withdraw,
                    })
                    .collect::<Vec<_>>();

                NetworkSpecs {
                    coin: coin.asset_code.clone(),
                    chains: chain_specs,
                }
            })
            .collect::<Vec<_>>()
    }
}

/*----- */
// Ticker info
/*----- */
#[derive(Debug, Deserialize)]
pub struct AscendExTickerInfo {
    pub code: u64,
    pub data: Vec<AscendExTickerInfoData>,
}

#[derive(Debug, Deserialize)]
pub struct AscendExTickerInfoData {
    pub symbol: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub domain: String,
    #[serde(rename = "tradingStartTime")]
    pub trading_start_time: u64,
    #[serde(rename = "collapseDecimals")]
    pub collapse_decimals: String,
    #[serde(rename = "minQty")]
    pub min_qty: String,
    #[serde(rename = "maxQty")]
    pub max_qty: String,
    #[serde(rename = "minNotional")]
    pub min_notional: String,
    #[serde(rename = "maxNotional")]
    pub max_notional: String,
    #[serde(rename = "statusCode")]
    pub status_code: String,
    #[serde(rename = "statusMessage")]
    pub status_message: String,
    #[serde(rename = "tickSize", deserialize_with = "de_str")]
    pub tick_size: f64,
    #[serde(rename = "useTick")]
    pub use_tick: bool,
    #[serde(rename = "lotSize")]
    pub lot_size: String,
    #[serde(rename = "useLot")]
    pub use_lot: bool,
    #[serde(rename = "commissionType")]
    pub commission_type: String,
    #[serde(rename = "commissionReserveRate")]
    pub commission_reserve_rate: String,
    #[serde(rename = "qtyScale")]
    pub qty_scale: i64,
    #[serde(rename = "priceScale")]
    pub price_scale: i64,
    #[serde(rename = "notionalScale")]
    pub notional_scale: i64,
}

impl From<AscendExTickerInfo> for TickerInfo {
    fn from(_value: AscendExTickerInfo) -> Self {
        unimplemented!()
    }
}
