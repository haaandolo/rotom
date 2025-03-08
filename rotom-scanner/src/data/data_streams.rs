use futures::StreamExt;
use rotom_data::{
    exchange::{
        binance::BinanceSpotPublicData, coinex::CoinExSpotPublicData, exmo::ExmoSpotPublicData,
        htx::HtxSpotPublicData, kucoin::KuCoinSpotPublicData, okx::OkxSpotPublicData,
        woox::WooxSpotPublicData,
    },
    model::{
        market_event::{DataKind, MarketEvent},
        network_info::NetworkSpecs,
    },
    shared::subscription_models::{ExchangeId, Instrument, StreamKind},
    streams::dynamic_stream::DynamicStreams,
};
use tokio::sync::mpsc;

use super::{network_status_stream::NetworkStatusStream, stream_chunk::StreamChunks};

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
        // .add_exchange::<BinanceSpotPublicData>(instruments.clone())
        .add_exchange::<ExmoSpotPublicData>(instruments.clone())
        .add_exchange::<HtxSpotPublicData>(instruments.clone())
        .add_exchange::<CoinExSpotPublicData>(instruments.clone())
        .add_exchange::<KuCoinSpotPublicData>(instruments.clone())
        .add_exchange::<OkxSpotPublicData>(instruments.clone())
        .add_exchange::<WooxSpotPublicData>(instruments.clone())
        .build();

    /*----- */
    // Stream chunk builder - todo change these awaits
    /*----- */
    let stream_init = StreamChunks::default()
        .add_exchange::<WooxSpotPublicData>()
        .await
        .add_exchange::<HtxSpotPublicData>()
        .await
        .add_exchange::<CoinExSpotPublicData>()
        .await
        .add_exchange::<ExmoSpotPublicData>()
        .await
        .add_exchange::<KuCoinSpotPublicData>()
        .await
        .add_exchange::<OkxSpotPublicData>()
        .await
        .build();

    /*----- */
    // Market data stream
    /*----- */
    // Binance - (trade, l2)
    // Htx - (trades, snapshot) <--- 50 chunks
    // Woo X (one ws conn per ticker) - (trade, snapshot) <--- 1 chunk
    // Coinex - (trades, snapshot) <--- 50 chunks
    // Okx - (trade,  snapshot)  <--- 50 chunks
    // Kucoin - (trade, snaphshot) <--- 50 chunks
    // Exmo - (Trades, Snapshot) <--- 50 chunks
    // Ascendex - (Trades, L2)
    // Phemex (one ws conn per ticker) - (Trades, L2)

    // network status - woox, kucoin, 
    
    /*------ */
    // Testing streams
    /*------ */
    // let streams = DynamicStreams::init([vec![
    //     // Binance
    //     (ExchangeId::OkxSpot, "ada", "usdt", StreamKind::Trade),
    //     (ExchangeId::OkxSpot, "ada", "usdt", StreamKind::Snapshot),
    //     (ExchangeId::OkxSpot, "icp", "usdt", StreamKind::Trade),
    //     (ExchangeId::OkxSpot, "icp", "usdt", StreamKind::Snapshot),
    //     (ExchangeId::OkxSpot, "sol", "usdt", StreamKind::Trade),
    //     (ExchangeId::OkxSpot, "sol", "usdt", StreamKind::Snapshot),
    //     // Htx
    //     (ExchangeId::HtxSpot, "ada", "usdt", StreamKind::Trades),
    //     (ExchangeId::HtxSpot, "ada", "usdt", StreamKind::Snapshot),
    //     (ExchangeId::HtxSpot, "icp", "usdt", StreamKind::Trades),
    //     (ExchangeId::HtxSpot, "icp", "usdt", StreamKind::Snapshot),
    //     (ExchangeId::HtxSpot, "sol", "usdt", StreamKind::Trades),
    //     (ExchangeId::HtxSpot, "sol", "usdt", StreamKind::Snapshot),
    //     // CoinEx
    //     (ExchangeId::KuCoinSpot, "ada", "usdt", StreamKind::Trade),
    //     (ExchangeId::KuCoinSpot, "ada", "usdt", StreamKind::Snapshot),
    //     (ExchangeId::KuCoinSpot, "icp", "usdt", StreamKind::Trade),
    //     (ExchangeId::KuCoinSpot, "icp", "usdt", StreamKind::Snapshot),
    //     (ExchangeId::KuCoinSpot, "sol", "usdt", StreamKind::Trade),
    //     (ExchangeId::KuCoinSpot, "sol", "usdt", StreamKind::Snapshot),
    // ]])
    // .await
    // .unwrap();

    let streams = DynamicStreams::init(stream_init).await.unwrap();
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
