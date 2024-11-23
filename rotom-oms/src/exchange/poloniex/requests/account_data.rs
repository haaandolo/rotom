use chrono::{DateTime, Utc};
use rotom_data::shared::de::de_str;
use rotom_data::shared::de::de_u64_epoch_ms_as_datetime_utc;
use rotom_data::shared::subscription_models::ExchangeId;
use serde::{Deserialize, Serialize};

use crate::model::balance::AssetBalance;
use crate::model::balance::Balance;
use crate::model::order::Open;
use crate::model::order::Order;
use crate::model::ClientOrderId;
use crate::model::Side;

use super::{PoloniexOrderStatus, PoloniexOrderType};

/*----- */
// Poloniex User Data - Orders
/*----- */
#[derive(Debug, Deserialize)]
pub struct PoloniexAccountDataOrder {
    pub channel: String,
    pub data: [PoloniexAccountDataOrderParams; 1],
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoloniexAccountDataOrderParams {
    pub symbol: String,
    pub r#type: PoloniexOrderType,
    #[serde(deserialize_with = "de_str")]
    pub quantity: f64,
    #[serde(deserialize_with = "de_str")]
    pub order_id: u64,
    #[serde(deserialize_with = "de_str")]
    pub trade_fee: f64,
    pub client_order_id: String,
    pub account_type: String,
    pub fee_currency: String,
    pub event_type: PoloniexOrderEventType,
    pub source: String,
    pub side: Side,
    #[serde(deserialize_with = "de_str")]
    pub filled_quantity: f64,
    #[serde(deserialize_with = "de_str")]
    pub filled_amount: f64,
    pub match_role: String,
    pub state: PoloniexOrderStatus,
    #[serde(deserialize_with = "de_u64_epoch_ms_as_datetime_utc")]
    pub trade_time: DateTime<Utc>,
    #[serde(deserialize_with = "de_str")]
    pub trade_amount: f64,
    #[serde(deserialize_with = "de_str")]
    pub order_amount: f64,
    #[serde(deserialize_with = "de_u64_epoch_ms_as_datetime_utc")]
    pub create_time: DateTime<Utc>,
    #[serde(deserialize_with = "de_str")]
    pub price: f64,
    #[serde(deserialize_with = "de_str")]
    pub trade_qty: f64,
    #[serde(deserialize_with = "de_str")]
    pub trade_price: f64,
    #[serde(deserialize_with = "de_str")]
    pub trade_id: u64,
    pub ts: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PoloniexOrderEventType {
    Place,
    Trade,
    Canceled,
}

// impl From<PoloniexAccountDataOrder> for Order<Open> {
//     fn from(mut order: PoloniexAccountDataOrder) -> Self {
//         Self {
//             exchange: ExchangeId::PoloniexSpot,
//             instrument: std::mem::take(&mut order.data[0].symbol),
//             client_order_id: ClientOrderId(std::mem::take(&mut order.data[0].client_order_id)),
//             side: order.data[0].side,
//             state: Open {
//                 id: order.data[0].order_id,
//                 price: order.data[0].price,
//                 quantity: order.data[0].quantity,
//                 filled_quantity: order.data[0].filled_amount,
//             },
//         }
//     }
// }

/*----- */
// Poloniex User Data - Balance
/*----- */
#[derive(Debug, Deserialize)]
pub struct PoloniexAccountDataBalance {
    pub channel: String,
    pub data: [PoloniexAccountDataBalanceParams; 1],
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoloniexAccountDataBalanceParams {
    pub change_time: u64,
    #[serde(deserialize_with = "de_str")]
    pub account_id: u64,
    #[serde(deserialize_with = "de_str")]
    pub account_type: String,
    pub event_type: PoloniexBalanceEventType,
    #[serde(deserialize_with = "de_str")]
    pub available: f64,
    pub currency: String,
    pub id: u64,
    pub user_id: u64,
    #[serde(deserialize_with = "de_str")]
    pub hold: f64,
    pub ts: u64,
}

impl From<PoloniexAccountDataBalance> for AssetBalance {
    fn from(mut account_balance: PoloniexAccountDataBalance) -> Self {
        Self {
            asset: std::mem::take(&mut account_balance.data[0].currency), // when changed to small string, can rm std::mem::take
            exchange: ExchangeId::PoloniexSpot,
            balance: Balance {
                total: account_balance.data[0].hold,
                available: account_balance.data[0].available,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "SCREAMING_SNAKE_CASE"))]
pub enum PoloniexBalanceEventType {
    PlaceOrder,
    CanceledOrder,
    MatchOrder,
    TransferIn,
    TransferOut,
    Deposit,
    Withdraw,
}

/*----- */
// Poloniex User Data Response
/*----- */
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum PoloniexAccountEvents {
    Order(PoloniexAccountDataOrder),
    Balance(PoloniexAccountDataBalance),
}

// impl From<PoloniexAccountEvents> for AccountData {
//     fn from(value: PoloniexAccountEvents) -> Self {
//         match value {
//             PoloniexAccountEvents::Order(order) => AccountData::from(order),
//             PoloniexAccountEvents::Balance(balance) => AccountData::from(balance),
//         }
//     }
// }
