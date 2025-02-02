use std::{
    io::{Read, Write},
    time::{SystemTime, UNIX_EPOCH},
};

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
            PhemexDeposit, PhemexOrderBookUpdate, PhemexSubscriptionResponse,
            PhemexTickerInfo, PhemexTradesUpdate, PhemexWithdraw,
        },
    },
    protocols::ws::ws_parser::{StreamParser, WebSocketParser},
    shared::de::de_str_u64_epoch_ms_as_datetime_utc,
};
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

/*
"{\"id\":10110522123195832575,\"status\":\"ok\",\"subbed\":\"market.btcusdt.mbp.refresh.5\",\"ts\":1738148629183}"
"{\"status\":\"error\",\"ts\":1738148682256,\"id\":17650732822015568887,\"err-code\":\"bad-request\",\"err-msg\":\"invalid command\"}"

conn success:

succuss:
Ok(Text("{\"error\":null,\"id\":0,\"result\":{\"status\":\"success\"}}"))

error:
Ok(Text("{\"error\":{\"code\":6001,\"message\":\"invalid argument\"},\"id\":null,\"result\":null}"))

Ok(Text("{\"book\":{\"asks\":[[10181527000000,191900],[10181528000000,1809700]],\"bids\":[[10172467000000,2670000],[10172466000000,98300]]},\"depth\":30,\"sequence\":36104719812,\"symbol\":\"sBTCUSDT\",\"timestamp\":1737948576197618371,\"type\":\"snapshot\"}"))
###########
Ok(Text("{\"book\":{\"asks\":[[10188178000000,0],[10188380000000,2779500]],\"bids\":[]},\"depth\":30,\"sequence\":36104719838,\"symbol\":\"sBTCUSDT\",\"timestamp\":1737948576636759086,\"type\":\"incremental\"}"))
Ok(Text("{\"book\":{\"asks\":[],\"bids\":[[10167822000000,1924300],[10160289000000,0]]},\"depth\":30,\"sequence\":36104719854,\"symbol\":\"sBTCUSDT\",\"timestamp\":1737948576649455801,\"type\":\"incremental\"}"))
Ok(Text("{\"book\":{\"asks\":[[10181528000000,0],[10181926000000,2516900],[10182089000000,9539400],[10211372000000,0]],\"bids\":[]},\"depth\":30,\"sequence\":36104719861,\"symbol\":\"sBTCUSDT\",\"timestamp\":1737948576926996935,\"type\":\"incremental\"}"))
Ok(Text("{\"book\":{\"asks\":[[10181527000000,0],[10211372000000,91000400]],\"bids\":[]},\"depth\":30,\"sequence\":36104719863,\"symbol\":\"sBTCUSDT\",\"timestamp\":1737948577008443430,\"type\":\"incremental\"}"))
Ok(Text("{\"book\":{\"asks\":[],\"bids\":[[10164491000000,2703700],[10164037000000,0],[10148563000000,191465500],[10143456000000,66871400],[10130482000000,0],[10130000000000,0],[10130482000000,0]]},\"depth\":30,\"sequence\":36104719871,\"symbol\":\"sBTCUSDT\",\"timestamp\":1737948577171477701,\"type\":\"incremental\"}"))
*/

/*----- */
// Test http
/*----- */
pub async fn test_http() {
    //curl --location ''

    // https://ascendex.com/api/pro/v1/depth?symbol=ASD/USDT

    let base_url = "https://api.phemex.com";
    // let request_path = "/exchange/public/cfg/chain-settings";
    let request_path = "/phemex-withdraw/wallets/api/asset/info";

    let url = format!("{}{}", base_url, request_path);
    // let url = format!("{}{}", base_url, request_path);
    let test = reqwest::get(url)
        .await
        .unwrap()
        .text()
        // .json::<PhemexNetworkInfo>()
        // .json::<serde_json::Value>()
        .await
        .unwrap();

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
    println!("{}",url);

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

// sign=CryptoJS.enc.Base64.stringify(
//    CryptoJS.HmacSHA256(timestamp + 'GET' + '/api/v5/account/balance?ccy=BTC', SecretKey)
// )
