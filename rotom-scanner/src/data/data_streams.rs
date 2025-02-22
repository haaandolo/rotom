use futures::StreamExt;
use rotom_data::{
    exchange::{
        binance::BinanceSpotPublicData, coinex::CoinExSpotPublicData, htx::HtxSpotPublicData,
        okx::OkxSpotPublicData, woox::WooxSpotPublicData,
    },
    model::{
        market_event::{DataKind, MarketEvent},
        network_info::NetworkSpecs,
    },
    shared::subscription_models::{ExchangeId, Instrument, StreamKind},
    streams::dynamic_stream::DynamicStreams,
};
use tokio::sync::mpsc;

use super::network_status_stream::NetworkStatusStream;

pub async fn get_spot_arb_data_streams() -> (
    mpsc::UnboundedReceiver<MarketEvent<DataKind>>,
    mpsc::UnboundedReceiver<NetworkSpecs>,
) {
    /*----- */
    // Declare instruments for each exchange
    /*----- */
    let instruments = vec![
        Instrument::new("btc", "usdt"),
        Instrument::new("eth", "usdt"),
        Instrument::new("ada", "usdt"),
        Instrument::new("sol", "usdt"),
        Instrument::new("icp", "usdt"),
    ];

    /*----- */
    // Network status stream
    /*----- */
    let network_stream = NetworkStatusStream::new()
        // .add_exchange::<AscendExSpotPublicData>(instruments.clone())
        .add_exchange::<BinanceSpotPublicData>(instruments.clone())
        // .add_exchange::<ExmoSpotPublicData>(instruments.clone())
        .add_exchange::<HtxSpotPublicData>(instruments.clone())
        // .add_exchange::<CoinExSpotPublicData>(instruments.clone())
        // .add_exchange::<KuCoinSpotPublicData>(instruments.clone())
        .add_exchange::<OkxSpotPublicData>(instruments.clone())
        // .add_exchange::<WooxSpotPublicData>(instruments.clone())
        .build();

    /*----- */
    // Market data stream
    /*----- */
    // Binance - (trade, l2)
    // Htx - (trades, snapshot) <---
    // Woo X (one ws conn per ticker) - (trade, snapshot) <---
    // Coinex - (trades, snapshot) <---
    // Okx - (trade,  snapshot)  <---
    // Kucoin - (trade, snaphshot) <---
    // Exmo - (Trades, Snapshot) <---
    // Ascendex - (Trades, L2)
    // Phemex (one ws conn per ticker) - (Trades, L2)

    let streams = DynamicStreams::init([
        /*----- */
        // Ascendex
        /*----- */
        vec![
            (ExchangeId::AscendExSpot, "zeta", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "zyn", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "maps", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "turbo", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "powsche", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "trumpsol", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "wtf", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "popdog", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "ath", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "beeg", "usdt", StreamKind::L2),
        ],
        vec![
            (ExchangeId::AscendExSpot, "bnbxbt", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "lester", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "doge", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "fight", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "zee", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "piin", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "move", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "syrup", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "niox", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "lfgo", "usdt", StreamKind::L2),
        ],
        vec![
            (ExchangeId::AscendExSpot, "spore", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "geeq", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "chz", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "fullsend", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "kango", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "xec", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "sqid", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "siren", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "deepseek", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "scuba", "usdt", StreamKind::L2),
        ],
        vec![
            (ExchangeId::AscendExSpot, "$cult", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "sophon", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "blur", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "ai", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "sfp", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "bets", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "bturbo", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "kekius", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "elonsol", "usdt", StreamKind::L2),
            (ExchangeId::AscendExSpot, "aixbt", "usdt", StreamKind::L2),
        ],
    ])
    .await
    .unwrap();

    let mut data = streams.select_all::<MarketEvent<DataKind>>();
    let (market_data_tx, market_data_rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        while let Some(event) = data.next().await {
            // println!("{:?}", event);
            let _ = market_data_tx.send(event);
        }
    });

    /*----- */
    // Return streams
    /*----- */
    (market_data_rx, network_stream)
}
