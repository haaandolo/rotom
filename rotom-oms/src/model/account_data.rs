use chrono::{DateTime, Utc};
use rotom_data::shared::subscription_models::ExchangeId;
use serde::Deserialize;

use super::{balance::Balance, Side};

/*----- */
// State
/*----- */
#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(rename_all(deserialize = "SCREAMING_SNAKE_CASE"))]
pub enum OrderStatus {
    New,
    Canceled,
    Rejected,
    Expired,
    PendingNew,
    PartiallyFilled,
    Filled,
    PendingCancel,
    ExpiredInMatch,
    PartiallyCanceled,
    Failed,
}

#[derive(Debug)]
pub struct AccountDataOrder {
    pub exchange: ExchangeId,
    pub asset: String,
    pub price: f64,
    pub quantity: f64,
    pub status: OrderStatus,
    pub execution_time: DateTime<Utc>,
    pub side: Side,
    pub fee: f64,
}

#[derive(Debug)]
pub struct AccountDataBalance {
    pub asset: String, // can be smolstr e.g. btc
    pub exchange: ExchangeId,
    pub balance: Balance,
}

#[derive(Debug)]
pub struct AccountDataBalanceDelta {
    pub asset: String,
    pub exchange: ExchangeId,
    pub total: f64,
    pub available: f64, // only used for margin will be zero if spot
}

#[derive(Debug)]
pub enum AccountData {
    Order(AccountDataOrder),
    BalanceVec(Vec<AccountDataBalance>),
    Balance(AccountDataBalance),
    BalanceDelta(AccountDataBalanceDelta),
}

/*
ExchangeOne(
    Order(
        BinanceAccountDataOrder {
            e: "executionReport",
            E: 1732732273708,
            s: "OPUSDT",
            c: "web_c2d6b53fbabd4f8ea828a58b055f729a",
            S: Buy,
            o: "LIMIT",
            f: GTC,
            q: 5.0,
            p: 1.0,
            P: 0.0,
            F: 0.0,
            g: -1,
            C: "",
            x: New,
            X: New,
            r: "NONE",
            i: 1860422927,
            l: 0.0,
            z: 0.0,
            L: 0.0,
            n: 0.0,
            N: None,
            T: 1732732273707,
            t: -1,
            v: None,
            I: 3838500896,
            w: true,
            m: false,
            M: false,
            O: 1732732273707,
            Z: 0.0,
            Y: 0.0,
            Q: 0.0,
            W: 1732732273707,
            V: "EXPIRE_MAKER",
        },
    ),
)
ExchangeOne(
    Balance(
        BinanceAccountDataBalance {
            e: "outboundAccountPosition",
            E: 1732732273708,
            u: 1732732273707,
            B: [
                BinanceAccountDataBalanceVec {
                    a: "BNB",
                    f: 0.0,
                    l: 0.0,
                },
                BinanceAccountDataBalanceVec {
                    a: "USDT",
                    f: 9.76940179,
                    l: 5.0,
                },
                BinanceAccountDataBalanceVec {
                    a: "OP",
                    f: 0.3115208,
                    l: 0.0,
                },
            ],
        },
    ),
)
ExchangeOne(
    Order(
        BinanceAccountDataOrder {
            e: "executionReport",
            E: 1732732307242,
            s: "OPUSDT",
            c: "web_3d679ceba4884287a4213adb10cebe9a",
            S: Buy,
            o: "LIMIT",
            f: GTC,
            q: 5.0,
            p: 1.0,
            P: 0.0,
            F: 0.0,
            g: -1,
            C: "web_c2d6b53fbabd4f8ea828a58b055f729a",
            x: Canceled,
            X: Canceled,
            r: "NONE",
            i: 1860422927,
            l: 0.0,
            z: 0.0,
            L: 0.0,
            n: 0.0,
            N: None,
            T: 1732732307241,
            t: -1,
            v: None,
            I: 3838504888,
            w: false,
            m: false,
            M: false,
            O: 1732732273707,
            Z: 0.0,
            Y: 0.0,
            Q: 0.0,
            W: 1732732273707,
            V: "EXPIRE_MAKER",
        },
    ),
)
ExchangeOne(
    Balance(
        BinanceAccountDataBalance {
            e: "outboundAccountPosition",
            E: 1732732307242,
            u: 1732732307241,
            B: [
                BinanceAccountDataBalanceVec {
                    a: "BNB",
                    f: 0.0,
                    l: 0.0,
                },
                BinanceAccountDataBalanceVec {
                    a: "USDT",
                    f: 14.76940179,
                    l: 0.0,
                },
                BinanceAccountDataBalanceVec {
                    a: "OP",
                    f: 0.3115208,
                    l: 0.0,
                },
            ],
        },
    ),
)
ExchangeOne(
    Order(
        BinanceAccountDataOrder {
            e: "executionReport",
            E: 1732732338935,
            s: "OPUSDT",
            c: "web_1afd770d26da4e2b910f6ad322483fd7",
            S: Buy,
            o: "MARKET",
            f: GTC,
            q: 2.13,
            p: 0.0,
            P: 0.0,
            F: 0.0,
            g: -1,
            C: "",
            x: New,
            X: New,
            r: "NONE",
            i: 1860427211,
            l: 0.0,
            z: 0.0,
            L: 0.0,
            n: 0.0,
            N: None,
            T: 1732732338935,
            t: -1,
            v: None,
            I: 3838509645,
            w: true,
            m: false,
            M: false,
            O: 1732732338935,
            Z: 0.0,
            Y: 0.0,
            Q: 5.0,
            W: 1732732338935,
            V: "EXPIRE_MAKER",
        },
    ),
)
ExchangeOne(
    Balance(
        BinanceAccountDataBalance {
            e: "outboundAccountPosition",
            E: 1732732338935,
            u: 1732732338935,
            B: [
                BinanceAccountDataBalanceVec {
                    a: "BNB",
                    f: 0.0,
                    l: 0.0,
                },
                BinanceAccountDataBalanceVec {
                    a: "USDT",
                    f: 9.79159179,
                    l: 0.0,
                },
                BinanceAccountDataBalanceVec {
                    a: "OP",
                    f: 2.4393908,
                    l: 0.0,
                },
            ],
        },
    ),
)
ExchangeOne(
    Order(
        BinanceAccountDataOrder {
            e: "executionReport",
            E: 1732732380135,
            s: "OPUSDT",
            c: "web_c4c2293ef5d24b708b622950b81cf249",
            S: Sell,
            o: "MARKET",
            f: GTC,
            q: 2.43,
            p: 0.0,
            P: 0.0,
            F: 0.0,
            g: -1,
            C: "",
            x: New,
            X: New,
            r: "NONE",
            i: 1860431288,
            l: 0.0,
            z: 0.0,
            L: 0.0,
            n: 0.0,
            N: None,
            T: 1732732380135,
            t: -1,
            v: None,
            I: 3838517956,
            w: true,
            m: false,
            M: false,
            O: 1732732380135,
            Z: 0.0,
            Y: 0.0,
            Q: 0.0,
            W: 1732732380135,
            V: "EXPIRE_MAKER",
        },
    ),
)
ExchangeOne(
    Balance(
        BinanceAccountDataBalance {
            e: "outboundAccountPosition",
            E: 1732732380135,
            u: 1732732380135,
            B: [
                BinanceAccountDataBalanceVec {
                    a: "BNB",
                    f: 0.0,
                    l: 0.0,
                },
                BinanceAccountDataBalanceVec {
                    a: "USDT",
                    f: 15.46239531,
                    l: 0.0,
                },
                BinanceAccountDataBalanceVec {
                    a: "OP",
                    f: 0.0093908,
                    l: 0.0,
                },
            ],
        },
    ),
)

### Poloniex ###
ExchangeTwo(
    Balance(
        PoloniexAccountDataBalance {
            channel: "balances",
            data: [
                PoloniexAccountDataBalanceParams {
                    change_time: 1732732407452,
                    account_id: 292264758130978818,
                    account_type: "SPOT",
                    event_type: PlaceOrder,
                    available: 3.86591388232,
                    currency: "USDT",
                    id: 384894532119977984,
                    user_id: 1887604,
                    hold: 5.0,
                    ts: 1732732407457,
                },
            ],
        },
    ),
)
ExchangeTwo(
    Order(
        PoloniexAccountDataOrder {
            channel: "orders",
            data: [
                PoloniexAccountDataOrderParams {
                    symbol: "OP_USDT",
                    type: Market,
                    quantity: 0.0,
                    order_id: 384894532069720064,
                    trade_fee: 0.0,
                    client_order_id: "",
                    account_type: "SPOT",
                    fee_currency: "",
                    event_type: Place,
                    source: "WEB",
                    side: Buy,
                    filled_quantity: 0.0,
                    filled_amount: 0.0,
                    match_role: "",
                    state: New,
                    trade_time: 1970-01-01T00:00:00Z,
                    trade_amount: 0.0,
                    order_amount: 5.0,
                    create_time: 2024-11-27T18:33:27.440Z,
                    price: 0.0,
                    trade_qty: 0.0,
                    trade_price: 0.0,
                    trade_id: 0,
                    ts: 1732732407473,
                },
            ],
        },
    ),
)
ExchangeTwo(
    Balance(
        PoloniexAccountDataBalance {
            channel: "balances",
            data: [
                PoloniexAccountDataBalanceParams {
                    change_time: 1732732407501,
                    account_id: 292264758130978818,
                    account_type: "SPOT",
                    event_type: MatchOrder,
                    available: 3.86591388232,
                    currency: "USDT",
                    id: 384894532325490689,
                    user_id: 1887604,
                    hold: 0.00014436,
                    ts: 1732732407510,
                },
            ],
        },
    ),
)
ExchangeTwo(
    Balance(
        PoloniexAccountDataBalance {
            channel: "balances",
            data: [
                PoloniexAccountDataBalanceParams {
                    change_time: 1732732407501,
                    account_id: 292264758130978818,
                    account_type: "SPOT",
                    event_type: MatchOrder,
                    available: 3.86605824232,
                    currency: "USDT",
                    id: 384894532325490690,
                    user_id: 1887604,
                    hold: 0.0,
                    ts: 1732732407510,
                },
            ],
        },
    ),
)
ExchangeTwo(
    Balance(
        PoloniexAccountDataBalance {
            channel: "balances",
            data: [
                PoloniexAccountDataBalanceParams {
                    change_time: 1732732407502,
                    account_id: 292264758130978818,
                    account_type: "SPOT",
                    event_type: MatchOrder,
                    available: 2.1202856,
                    currency: "OP",
                    id: 384894532329684993,
                    user_id: 1887604,
                    hold: 0.0,
                    ts: 1732732407510,
                },
            ],
        },
    ),
)
ExchangeTwo(
    Balance(
        PoloniexAccountDataBalance {
            channel: "balances",
            data: [
                PoloniexAccountDataBalanceParams {
                    change_time: 1732732407502,
                    account_id: 292264758130978818,
                    account_type: "SPOT",
                    event_type: MatchOrder,
                    available: 2.1160452,
                    currency: "OP",
                    id: 384894532329684994,
                    user_id: 1887604,
                    hold: 0.0,
                    ts: 1732732407510,
                },
            ],
        },
    ),
)
ExchangeTwo(
    Order(
        PoloniexAccountDataOrder {
            channel: "orders",
            data: [
                PoloniexAccountDataOrderParams {
                    symbol: "OP_USDT",
                    type: Market,
                    quantity: 0.0,
                    order_id: 384894532069720064,
                    trade_fee: 0.0042404,
                    client_order_id: "",
                    account_type: "SPOT",
                    fee_currency: "OP",
                    event_type: Trade,
                    source: "WEB",
                    side: Buy,
                    filled_quantity: 2.1202,
                    filled_amount: 4.99985564,
                    match_role: "TAKER",
                    state: Filled,
                    trade_time: 2024-11-27T18:33:27.463Z,
                    trade_amount: 4.99985564,
                    order_amount: 5.0,
                    create_time: 2024-11-27T18:33:27.440Z,
                    price: 0.0,
                    trade_qty: 2.1202,
                    trade_price: 2.3582,
                    trade_id: 99599839,
                    ts: 1732732407519,
                },
            ],
        },
    ),
)
ExchangeTwo(
    Balance(
        PoloniexAccountDataBalance {
            channel: "balances",
            data: [
                PoloniexAccountDataBalanceParams {
                    change_time: 1732732422430,
                    account_id: 292264758130978818,
                    account_type: "SPOT",
                    event_type: PlaceOrder,
                    available: 4.52e-5,
                    currency: "OP",
                    id: 384894594942279682,
                    user_id: 1887604,
                    hold: 2.116,
                    ts: 1732732422435,
                },
            ],
        },
    ),
)
ExchangeTwo(
    Order(
        PoloniexAccountDataOrder {
            channel: "orders",
            data: [
                PoloniexAccountDataOrderParams {
                    symbol: "OP_USDT",
                    type: Market,
                    quantity: 2.116,
                    order_id: 384894594896191488,
                    trade_fee: 0.0,
                    client_order_id: "",
                    account_type: "SPOT",
                    fee_currency: "",
                    event_type: Place,
                    source: "WEB",
                    side: Sell,
                    filled_quantity: 0.0,
                    filled_amount: 0.0,
                    match_role: "",
                    state: New,
                    trade_time: 1970-01-01T00:00:00Z,
                    trade_amount: 0.0,
                    order_amount: 0.0,
                    create_time: 2024-11-27T18:33:42.419Z,
                    price: 0.0,
                    trade_qty: 0.0,
                    trade_price: 0.0,
                    trade_id: 0,
                    ts: 1732732422450,
                },
            ],
        },
    ),
)
ExchangeTwo(
    Balance(
        PoloniexAccountDataBalance {
            channel: "balances",
            data: [
                PoloniexAccountDataBalanceParams {
                    change_time: 1732732422464,
                    account_id: 292264758130978818,
                    account_type: "SPOT",
                    event_type: MatchOrder,
                    available: 8.85473984232,
                    currency: "USDT",
                    id: 384894595084869632,
                    user_id: 1887604,
                    hold: 0.0,
                    ts: 1732732422470,
                },
            ],
        },
    ),
)
ExchangeTwo(
    Balance(
        PoloniexAccountDataBalance {
            channel: "balances",
            data: [
                PoloniexAccountDataBalanceParams {
                    change_time: 1732732422464,
                    account_id: 292264758130978818,
                    account_type: "SPOT",
                    event_type: MatchOrder,
                    available: 8.84476247912,
                    currency: "USDT",
                    id: 384894595084869633,
                    user_id: 1887604,
                    hold: 0.0,
                    ts: 1732732422470,
                },
            ],
        },
    ),
)
ExchangeTwo(
    Balance(
        PoloniexAccountDataBalance {
            channel: "balances",
            data: [
                PoloniexAccountDataBalanceParams {
                    change_time: 1732732422466,
                    account_id: 292264758130978818,
                    account_type: "SPOT",
                    event_type: MatchOrder,
                    available: 4.52e-5,
                    currency: "OP",
                    id: 384894595093258241,
                    user_id: 1887604,
                    hold: 0.0,
                    ts: 1732732422470,
                },
            ],
        },
    ),
)
ExchangeTwo(
    Order(
        PoloniexAccountDataOrder {
            channel: "orders",
            data: [
                PoloniexAccountDataOrderParams {
                    symbol: "OP_USDT",
                    type: Market,
                    quantity: 2.116,
                    order_id: 384894594896191488,
                    trade_fee: 0.0099773632,
                    client_order_id: "",
                    account_type: "SPOT",
                    fee_currency: "USDT",
                    event_type: Trade,
                    source: "WEB",
                    side: Sell,
                    filled_quantity: 2.116,
                    filled_amount: 4.9886816,
                    match_role: "TAKER",
                    state: Filled,
                    trade_time: 2024-11-27T18:33:42.439Z,
                    trade_amount: 4.9886816,
                    order_amount: 0.0,
                    create_time: 2024-11-27T18:33:42.419Z,
                    price: 0.0,
                    trade_qty: 2.116,
                    trade_price: 2.3576,
                    trade_id: 99599885,
                    ts: 1732732422480,
                },
            ],
        },
    ),
)

ExchangeTwo(
    Balance(
        PoloniexAccountDataBalance {
            channel: "balances",
            data: [
                PoloniexAccountDataBalanceParams {
                    change_time: 1732734285729,
                    account_id: 292264758130978818,
                    account_type: "SPOT",
                    event_type: PlaceOrder,
                    available: 3.84476247912,
                    currency: "USDT",
                    id: 384902410184720384,
                    user_id: 1887604,
                    hold: 5.0,
                    ts: 1732734285733,
                },
            ],
        },
    ),
)
ExchangeTwo(
    Order(
        PoloniexAccountDataOrder {
            channel: "orders",
            data: [
                PoloniexAccountDataOrderParams {
                    symbol: "OP_USDT",
                    type: Limit,
                    quantity: 5.0,
                    order_id: 384902410130255872,
                    trade_fee: 0.0,
                    client_order_id: "",
                    account_type: "SPOT",
                    fee_currency: "",
                    event_type: Place,
                    source: "WEB",
                    side: Buy,
                    filled_quantity: 0.0,
                    filled_amount: 0.0,
                    match_role: "",
                    state: New,
                    trade_time: 1970-01-01T00:00:00Z,
                    trade_amount: 0.0,
                    order_amount: 0.0,
                    create_time: 2024-11-27T19:04:45.716Z,
                    price: 1.0,
                    trade_qty: 0.0,
                    trade_price: 0.0,
                    trade_id: 0,
                    ts: 1732734285744,
                },
            ],
        },
    ),
)
ExchangeTwo(
    Balance(
        PoloniexAccountDataBalance {
            channel: "balances",
            data: [
                PoloniexAccountDataBalanceParams {
                    change_time: 1732734295694,
                    account_id: 292264758130978818,
                    account_type: "SPOT",
                    event_type: CanceledOrder,
                    available: 8.84476247912,
                    currency: "USDT",
                    id: 384902451980959745,
                    user_id: 1887604,
                    hold: 0.0,
                    ts: 1732734295700,
                },
            ],
        },
    ),
)
ExchangeTwo(
    Order(
        PoloniexAccountDataOrder {
            channel: "orders",
            data: [
                PoloniexAccountDataOrderParams {
                    symbol: "OP_USDT",
                    type: Limit,
                    quantity: 5.0,
                    order_id: 384902410130255872,
                    trade_fee: 0.0,
                    client_order_id: "",
                    account_type: "SPOT",
                    fee_currency: "",
                    event_type: Canceled,
                    source: "WEB",
                    side: Buy,
                    filled_quantity: 0.0,
                    filled_amount: 0.0,
                    match_role: "",
                    state: Canceled,
                    trade_time: 1970-01-01T00:00:00Z,
                    trade_amount: 0.0,
                    order_amount: 0.0,
                    create_time: 2024-11-27T19:04:45.716Z,
                    price: 1.0,
                    trade_qty: 0.0,
                    trade_price: 0.0,
                    trade_id: 0,
                    ts: 1732734295712,
                },
            ],
        },
    ),
)
*/
