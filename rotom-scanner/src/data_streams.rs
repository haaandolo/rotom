use futures::StreamExt;
use rotom_data::{
    exchange::binance::BinanceSpotPublicData,
    model::{
        market_event::{DataKind, MarketEvent},
        network_info::NetworkSpecs,
    },
    shared::subscription_models::{ExchangeId, Instrument, StreamKind},
    streams::dynamic_stream::DynamicStreams,
};
use tokio::sync::mpsc;

use crate::network_status_stream::NetworkStatusStream;

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
        // .add_exchange::<HtxSpotPublicData>(instruments.clone())
        // .add_exchange::<KuCoinSpotPublicData>(instruments.clone())
        // .add_exchange::<OkxSpotPublicData>(instruments.clone())
        // .add_exchange::<WooxSpotPublicData>(instruments.clone())
        .build();

    /*----- */
    // Market data stream
    /*----- */
    let streams = DynamicStreams::init([vec![
        (ExchangeId::BinanceSpot, "ada", "usdt", StreamKind::Trade),
        // (ExchangeId::BinanceSpot, "ada", "usdt", StreamKind::L2),
        // (ExchangeId::BinanceSpot, "icp", "usdt", StreamKind::Trade),
        // (ExchangeId::BinanceSpot, "icp", "usdt", StreamKind::L2),
        // (ExchangeId::BinanceSpot, "sol", "usdt", StreamKind::Trade),
        // (ExchangeId::BinanceSpot, "sol", "usdt", StreamKind::L2),
    ]])
    .await
    .unwrap();

    let mut data = streams.select_all::<MarketEvent<DataKind>>();
    let (market_data_tx, market_data_rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        while let Some(event) = data.next().await {
            println!("{:?}", event);
            let _ = market_data_tx.send(event);
        }
    });

    /*----- */
    // Return streams
    /*----- */
    (market_data_rx, network_stream)
}
