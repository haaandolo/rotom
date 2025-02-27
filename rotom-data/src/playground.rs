#[allow(unused_imports)]
use std::{
    io::{Read, Write},
    time::{SystemTime, UNIX_EPOCH},
};

#[allow(unused_imports)]
use crate::{
    exchange::{
        ascendex::model::{
            AscendExBookUpdate, AscendExNetworkInfo, AscendExOrderBookSnapshot,
            AscendExSubscriptionResponse, AscendExTickerInfo, AscendExTrades,
        },
        bitstamp::model::{BitstampOrderBookSnapshot, BitstampSubscriptionResponse, BitstampTrade},
        coinex::model::{CoinExNetworkInfo, CoinExOrderBookSnapshot, CoinExTrade},
        exmo::model::{ExmoOrderBookSnapshot, ExmoSubscriptionResponse, ExmoTrades},
        htx::model::HtxSubscriptionResponse,
        kucoin::model::{KuCoinNetworkInfo, KuCoinOrderBookSnapshot, KuCoinTrade, KuCoinWsUrl},
        okx::model::{OkxNetworkInfo, OkxOrderBookSnapshot, OkxSubscriptionResponse, OkxTrade},
        phemex::model::{
            PhemexDeposit, PhemexOrderBookUpdate, PhemexSubscriptionResponse, PhemexTickerInfo,
            PhemexTradesUpdate, PhemexWithdraw,
        },
    },
    protocols::ws::ws_parser::{StreamParser, WebSocketParser},
    shared::de::de_str_u64_epoch_ms_as_datetime_utc,
};

#[allow(unused_imports)]
use base64::{engine::general_purpose, Engine};
use chrono::{DateTime, Utc};
use flate2::read::GzDecoder;
use futures::{SinkExt, StreamExt};
use hmac::{Hmac, Mac};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Sha256;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[allow(unused_imports)]
use crate::{
    assets::level::Level,
    exchange::woox::model::{WooxNetworkInfo, WooxSubscriptionResponse, WooxTrade},
    protocols::ws::{schedule_pings_to_exchange, PingInterval},
};

/*----- */
// Test Ws
/*----- */
pub async fn test_ws() {
    let url = "wss://api-aws.huobi.pro/ws";

    let payload = json!({
      "id": rand::thread_rng().gen::<u64>(),
      "sub": vec![
        "market.btcusdt.mbp.refresh.5",
        "market.ethusdt.mbp.refresh.5",
        "market.adausdt.mbp.refresh.5",
      ]
    });

    // request: {"id":15354615839961262136,"method":"orderbook.subscribe","params":["sBTCUSDT"]}
    let (ws_stream, _) = connect_async(url).await.unwrap();
    let (mut write, mut read) = ws_stream.split();
    let _ = write.send(Message::text(payload.to_string())).await;

    // let ping_message = PingInterval {
    //     time: 500,
    //     message: json!({ "id":  uuid::Uuid::new_v4(), "type": "ping"}),
    // };

    // request: {"id":63801,"method":"subscribe","topics":["spot/order_book_snapshots:BTC_USDT","spot/order_book_snapshots:ETH_USDT"]}

    // tokio::spawn(schedule_pings_to_exchange(write, ping_message));

    while let Some(msg) = read.next().await {
        // println!("{:?}", msg);
        println!("###########");

        let test = WebSocketParser::parse::<HtxSubscriptionResponse>(msg);
        println!("{:?}", test);

        // if let Message::Binary(bin) = msg.unwrap() {
        //     let mut decoder = GzDecoder::new(&bin[..]);
        //     let mut decoded = String::new();

        //     let test = decoder.read_to_string(&mut decoded);
        //     println!("{:?}", decoded);
        // }
    }
}

/*----- */
// Test http
/*----- */
pub async fn test_http() {

    // curl -X GET https://ascendex.com/api/pro/v2/risk-limit-info"

    let base_url = "https://ascendex.com";
    let request_path = "/api/pro/v2/risk-limit-info";
    let url = format!("{}{}", base_url, request_path);

    let test = reqwest::get(url)
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();

    // let test = test["rows"]
    //     .as_array()
    //     .unwrap()
    //     .iter()
    //     .filter_map(|ticker| {
    //         let status = ticker["status"].as_str().unwrap().to_string();
    //         let symbol = ticker["symbol"].as_str().unwrap().to_lowercase();

    //         let mut ticker_split = symbol.split("_"); // comes like SPOT_BTC_USDT or PERP_BTC_USDT
    //         let ticker_kind = ticker_split.next().unwrap_or("").to_string();
    //         let base = ticker_split.next().unwrap_or("").to_string();
    //         let quote = ticker_split.next().unwrap_or("").to_string();

    //         if ticker_kind == "spot" && status == "TRADING" && quote == "usdt" {
    //             Some((base, quote))
    //         } else {
    //             None
    //         }
    //     })
    //     .collect::<Vec<_>>();

    println!("{:#?}", test);

    // let mut file = std::fs::File::create("./temppp.txt").unwrap();
    // file.write_all(test.as_bytes());
}

pub async fn test_http_private() {
    let secret = env!("PHEMEX_API_SECRET");
    let key = env!("PHEMEX_API_KEY");
    let url = "https://api.phemex.com";
    let curr = "currency=BTC";

    let expiry = Utc::now().timestamp() as u64 + 60;

    // let request_path = "/phemex-withdraw/wallets/api/asset/info";
    let request_path = "/phemex-deposit/wallets/api/chainCfg";

    let sign_message = format!("{}{}{}", request_path, curr, expiry);

    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(sign_message.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());

    let url = format!("{}{}?{}", url, request_path, curr);
    println!("{}", url);

    let test = reqwest::Client::new()
        .get(url)
        .header("x-phemex-access-token", key)
        .header("x-phemex-request-signature", signature)
        .header("x-phemex-request-expiry", expiry.to_string())
        .send()
        .await
        .unwrap()
        // .json::<PhemexWithdraw>()
        .json::<PhemexDeposit>()
        // .json::<serde_json::Value>()
        // .text()
        .await
        .unwrap();

    println!("{:#?}", test);
}


/*
// before
Object {
    "code": Number(0),
    "data": Object {
        "ip": String("115.188.127.241"),
        "webSocket": Object {
            "limits": Object {
                "maxWebSocketSessionsPerIpAccountGroup": Number(20),
                "maxWebSocketSessionsPerIpTotal": Number(300),
            },
            "messageThreshold": Object {
                "level1OpThreshold": Object {
                    "auth": Number(800),
                    "ping": Number(800),
                    "pong": Number(800),
                    "req": Number(10000),
                    "sub": Number(150),
                    "unsub": Number(150),
                },
                "level1ReqThreshold": Object {
                    "balance": Number(10000),
                    "batch_cancel_order": Number(10000),
                    "batch_place_order": Number(10000),
                    "cancel_all": Number(3000),
                    "cancel_order": Number(3000),
                    "depth_snapshot": Number(300),
                    "depth_snapshot_top100": Number(300),
                    "futures_account_snapshot": Number(10000),
                    "futures_open_orders": Number(10000),
                    "margin_risk": Number(10000),
                    "market_trades": Number(10000),
                    "open_order": Number(10000),
                    "place_order": Number(3000),
                },
                "level2OpThreshold": Object {
                    "auth": Number(1000),
                    "ping": Number(1000),
                    "pong": Number(1000),
                    "req": Number(10000),
                    "sub": Number(200),
                    "unsub": Number(200),
                },
                "level2ReqThreshold": Object {
                    "balance": Number(10000),
                    "batch_cancel_order": Number(10000),
                    "batch_place_order": Number(10000),
                    "cancel_all": Number(10000),
                    "cancel_order": Number(10000),
                    "depth_snapshot": Number(400),
                    "depth_snapshot_top100": Number(400),
                    "futures_account_snapshot": Number(10000),
                    "futures_open_orders": Number(10000),
                    "margin_risk": Number(10000),
                    "market_trades": Number(10000),
                    "open_order": Number(10000),
                    "place_order": Number(10000),
                },
            },
            "status": Object {
                "bannedUntil": Number(-1),
                "isBanned": Bool(false),
                "reason": String(""),
                "violationCode": Number(0),
            },
        },
    },
}

// after
Object {
    "code": Number(0),
    "data": Object {
        "ip": String("115.188.127.241"),
        "webSocket": Object {
            "limits": Object {
                "maxWebSocketSessionsPerIpAccountGroup": Number(20),
                "maxWebSocketSessionsPerIpTotal": Number(300),
            },
            "messageThreshold": Object {
                "level1OpThreshold": Object {
                    "auth": Number(800),
                    "ping": Number(800),
                    "pong": Number(800),
                    "req": Number(10000),
                    "sub": Number(150),
                    "unsub": Number(150),
                },
                "level1ReqThreshold": Object {
                    "balance": Number(10000),
                    "batch_cancel_order": Number(10000),
                    "batch_place_order": Number(10000),
                    "cancel_all": Number(3000),
                    "cancel_order": Number(3000),
                    "depth_snapshot": Number(300),
                    "depth_snapshot_top100": Number(300),
                    "futures_account_snapshot": Number(10000),
                    "futures_open_orders": Number(10000),
                    "margin_risk": Number(10000),
                    "market_trades": Number(10000),
                    "open_order": Number(10000),
                    "place_order": Number(3000),
                },
                "level2OpThreshold": Object {
                    "auth": Number(1000),
                    "ping": Number(1000),
                    "pong": Number(1000),
                    "req": Number(10000),
                    "sub": Number(200),
                    "unsub": Number(200),
                },
                "level2ReqThreshold": Object {
                    "balance": Number(10000),
                    "batch_cancel_order": Number(10000),
                    "batch_place_order": Number(10000),
                    "cancel_all": Number(10000),
                    "cancel_order": Number(10000),
                    "depth_snapshot": Number(400),
                    "depth_snapshot_top100": Number(400),
                    "futures_account_snapshot": Number(10000),
                    "futures_open_orders": Number(10000),
                    "margin_risk": Number(10000),
                    "market_trades": Number(10000),
                    "open_order": Number(10000),
                    "place_order": Number(10000),
                },
            },
            "status": Object {
                "bannedUntil": Number(-1),
                "isBanned": Bool(false),
                "reason": String(""),
                "violationCode": Number(0),
            },
        },
    },
}
*/