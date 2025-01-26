use futures::{
    stream::{select_all, SelectAll},
    Stream, StreamExt,
};
use itertools::Itertools;
use std::{collections::HashMap, fmt::Debug};
use tokio_stream::wrappers::UnboundedReceiverStream;
use vecmap::VecMap;

use super::single::ExchangeChannel;
use crate::{
    error::SocketError,
    exchange::{
        binance::BinanceSpotPublicData, bitstamp::BitstampSpotPublicData,
        coinex::CoinExSpotPublicData, exmo::ExmoSpotPublicData, htx::HtxSpotPublicData,
        kucoin::KuCoinSpotPublicData, okx::OkxSpotPublicData, poloniex::PoloniexSpotPublicData,
        woox::WooxSpotPublicData,
    },
    model::{
        event_book::{EventOrderBook, OrderBookL2},
        event_book_snapshot::{EventOrderBookSnapshot, OrderBookSnapshot},
        event_trade::{AggTrades, EventTrade, Trade, Trades},
        market_event::MarketEvent,
    },
    shared::subscription_models::{ExchangeId, StreamKind, Subscription},
    streams::consumer::consume,
};

/*----- */
// Dynamic Streams
/*----- */
#[derive(Debug)]
pub struct DynamicStreams {
    pub trade: VecMap<ExchangeId, UnboundedReceiverStream<MarketEvent<EventTrade>>>,
    pub trades: VecMap<ExchangeId, UnboundedReceiverStream<MarketEvent<Vec<EventTrade>>>>,
    pub l2s: VecMap<ExchangeId, UnboundedReceiverStream<MarketEvent<EventOrderBook>>>,
    pub snapshots: VecMap<ExchangeId, UnboundedReceiverStream<MarketEvent<EventOrderBookSnapshot>>>,
}

impl DynamicStreams {
    pub async fn init<SubBatchIter, SubIter, Sub>(
        subscription_batchs: SubBatchIter,
    ) -> Result<Self, SocketError>
    where
        SubBatchIter: IntoIterator<Item = SubIter>,
        SubIter: IntoIterator<Item = Sub> + Debug,
        Sub: Into<Subscription<ExchangeId, StreamKind>>,
    {
        let mut channels = Channels::default();
        for batch in subscription_batchs {
            // Convert to Subscriptions struct
            let mut exchange_sub = batch
                .into_iter()
                .map(Sub::into)
                .collect::<Vec<Subscription<_, _>>>();

            // Remove duplicates
            exchange_sub.sort();
            exchange_sub.dedup();

            // Group batches by exchange and stream kind
            let grouped = exchange_sub
                .into_iter()
                .chunk_by(|sub| (sub.exchange, sub.stream_kind));

            // Spawn the releveant streams for a specific exchange
            for ((exchange, stream_kind), subs) in grouped.into_iter() {
                match (exchange, stream_kind) {
                    /*----- */
                    // Binance Spot
                    /*----- */
                    (ExchangeId::BinanceSpot, StreamKind::L2) => {
                        tokio::spawn(consume::<BinanceSpotPublicData, OrderBookL2>(
                            subs.into_iter()
                                .map(|sub| {
                                    Subscription::new(
                                        BinanceSpotPublicData,
                                        sub.instrument,
                                        OrderBookL2,
                                    )
                                })
                                .collect(),
                            channels.l2s.entry(exchange).or_default().tx.clone(),
                        ));
                    }
                    (ExchangeId::BinanceSpot, StreamKind::Trade) => {
                        tokio::spawn(consume::<BinanceSpotPublicData, Trade>(
                            subs.into_iter()
                                .map(|sub| {
                                    Subscription::new(BinanceSpotPublicData, sub.instrument, Trade)
                                })
                                .collect(),
                            channels.trade.entry(exchange).or_default().tx.clone(),
                        ));
                    }
                    (ExchangeId::BinanceSpot, StreamKind::AggTrades) => {
                        tokio::spawn(consume::<BinanceSpotPublicData, AggTrades>(
                            subs.into_iter()
                                .map(|sub| {
                                    Subscription::new(
                                        BinanceSpotPublicData,
                                        sub.instrument,
                                        AggTrades,
                                    )
                                })
                                .collect(),
                            channels.trade.entry(exchange).or_default().tx.clone(),
                        ));
                    }
                    (ExchangeId::BinanceSpot, StreamKind::Snapshot) => {
                        unimplemented!()
                    }
                    (ExchangeId::BinanceSpot, StreamKind::Trades) => {
                        unimplemented!("Binance does not send multiple trades for the same symbol in one go like htx")
                    }
                    /*----- */
                    // Poloniex Spot
                    /*----- */
                    (ExchangeId::PoloniexSpot, StreamKind::L2) => {
                        tokio::spawn(consume::<PoloniexSpotPublicData, OrderBookL2>(
                            subs.into_iter()
                                .map(|sub| {
                                    Subscription::new(
                                        PoloniexSpotPublicData,
                                        sub.instrument,
                                        OrderBookL2,
                                    )
                                })
                                .collect(),
                            channels.l2s.entry(exchange).or_default().tx.clone(),
                        ));
                    }
                    (ExchangeId::PoloniexSpot, StreamKind::Trade) => {
                        tokio::spawn(consume::<PoloniexSpotPublicData, Trade>(
                            subs.into_iter()
                                .map(|sub| {
                                    Subscription::new(PoloniexSpotPublicData, sub.instrument, Trade)
                                })
                                .collect(),
                            channels.trade.entry(exchange).or_default().tx.clone(),
                        ));
                    }
                    // Poloniex's does not separate regular and aggregated trades
                    (ExchangeId::PoloniexSpot, StreamKind::AggTrades) => {
                        unimplemented!()
                    }
                    (ExchangeId::PoloniexSpot, StreamKind::Snapshot) => {
                        unimplemented!()
                    }
                    (ExchangeId::PoloniexSpot, StreamKind::Trades) => {
                        unimplemented!("Poloniex does not send multiple trades for the same symbol in one go like htx")
                    }
                    /*----- */
                    // Htx Spot
                    /*----- */
                    (ExchangeId::HtxSpot, StreamKind::Trades) => {
                        tokio::spawn(consume::<HtxSpotPublicData, Trades>(
                            subs.into_iter()
                                .map(|sub| {
                                    Subscription::new(HtxSpotPublicData, sub.instrument, Trades)
                                })
                                .collect(),
                            channels.trades.entry(exchange).or_default().tx.clone(),
                        ));
                    }
                    (ExchangeId::HtxSpot, StreamKind::Snapshot) => {
                        tokio::spawn(consume::<HtxSpotPublicData, OrderBookSnapshot>(
                            subs.into_iter()
                                .map(|sub| {
                                    Subscription::new(
                                        HtxSpotPublicData,
                                        sub.instrument,
                                        OrderBookSnapshot,
                                    )
                                })
                                .collect(),
                            channels.snapshots.entry(exchange).or_default().tx.clone(),
                        ));
                    }
                    (ExchangeId::HtxSpot, StreamKind::L2) => {
                        unimplemented!()
                    }
                    (ExchangeId::HtxSpot, StreamKind::AggTrades) => {
                        unimplemented!()
                    }
                    (ExchangeId::HtxSpot, StreamKind::Trade) => {
                        unimplemented!("Htx only sends trades aggregated")
                    }
                    /*----- */
                    // Woox Spot
                    /*----- */
                    (ExchangeId::WooxSpot, StreamKind::Snapshot) => {
                        tokio::spawn(consume::<WooxSpotPublicData, OrderBookSnapshot>(
                            subs.into_iter()
                                .map(|sub| {
                                    Subscription::new(
                                        WooxSpotPublicData,
                                        sub.instrument,
                                        OrderBookSnapshot,
                                    )
                                })
                                .collect(),
                            channels.snapshots.entry(exchange).or_default().tx.clone(),
                        ));
                    }
                    (ExchangeId::WooxSpot, StreamKind::Trade) => {
                        tokio::spawn(consume::<WooxSpotPublicData, Trade>(
                            subs.into_iter()
                                .map(|sub| {
                                    Subscription::new(WooxSpotPublicData, sub.instrument, Trade)
                                })
                                .collect(),
                            channels.trade.entry(exchange).or_default().tx.clone(),
                        ));
                    }
                    (ExchangeId::WooxSpot, StreamKind::Trades) => {}
                    (ExchangeId::WooxSpot, StreamKind::L2) => {
                        unimplemented!()
                    }
                    (ExchangeId::WooxSpot, StreamKind::AggTrades) => {
                        unimplemented!()
                    }
                    /*----- */
                    // Bitstamp spot
                    /*----- */
                    (ExchangeId::BitstampSpot, StreamKind::Snapshot) => {
                        tokio::spawn(consume::<BitstampSpotPublicData, OrderBookSnapshot>(
                            subs.into_iter()
                                .map(|sub| {
                                    Subscription::new(
                                        BitstampSpotPublicData,
                                        sub.instrument,
                                        OrderBookSnapshot,
                                    )
                                })
                                .collect(),
                            channels.snapshots.entry(exchange).or_default().tx.clone(),
                        ));
                    }
                    (ExchangeId::BitstampSpot, StreamKind::Trade) => {
                        tokio::spawn(consume::<BitstampSpotPublicData, Trade>(
                            subs.into_iter()
                                .map(|sub| {
                                    Subscription::new(BitstampSpotPublicData, sub.instrument, Trade)
                                })
                                .collect(),
                            channels.trade.entry(exchange).or_default().tx.clone(),
                        ));
                    }
                    (ExchangeId::BitstampSpot, StreamKind::Trades) => {
                        unimplemented!()
                    }
                    (ExchangeId::BitstampSpot, StreamKind::L2) => {
                        unimplemented!()
                    }
                    (ExchangeId::BitstampSpot, StreamKind::AggTrades) => {
                        unimplemented!()
                    }
                    /*----- */
                    // CoinEx spot
                    /*----- */
                    (ExchangeId::CoinExSpot, StreamKind::Snapshot) => {
                        tokio::spawn(consume::<CoinExSpotPublicData, OrderBookSnapshot>(
                            subs.into_iter()
                                .map(|sub| {
                                    Subscription::new(
                                        CoinExSpotPublicData,
                                        sub.instrument,
                                        OrderBookSnapshot,
                                    )
                                })
                                .collect(),
                            channels.snapshots.entry(exchange).or_default().tx.clone(),
                        ));
                    }
                    (ExchangeId::CoinExSpot, StreamKind::Trades) => {
                        tokio::spawn(consume::<CoinExSpotPublicData, Trades>(
                            subs.into_iter()
                                .map(|sub| {
                                    Subscription::new(CoinExSpotPublicData, sub.instrument, Trades)
                                })
                                .collect(),
                            channels.trades.entry(exchange).or_default().tx.clone(),
                        ));
                    }
                    (ExchangeId::CoinExSpot, StreamKind::Trade) => {
                        unimplemented!()
                    }
                    (ExchangeId::CoinExSpot, StreamKind::L2) => {
                        unimplemented!()
                    }
                    (ExchangeId::CoinExSpot, StreamKind::AggTrades) => {
                        unimplemented!()
                    }
                    /*----- */
                    // Okx Spot
                    /*----- */
                    (ExchangeId::OkxSpot, StreamKind::Snapshot) => {
                        tokio::spawn(consume::<OkxSpotPublicData, OrderBookSnapshot>(
                            subs.into_iter()
                                .map(|sub| {
                                    Subscription::new(
                                        OkxSpotPublicData,
                                        sub.instrument,
                                        OrderBookSnapshot,
                                    )
                                })
                                .collect(),
                            channels.snapshots.entry(exchange).or_default().tx.clone(),
                        ));
                    }
                    (ExchangeId::OkxSpot, StreamKind::Trade) => {
                        tokio::spawn(consume::<OkxSpotPublicData, Trade>(
                            subs.into_iter()
                                .map(|sub| {
                                    Subscription::new(OkxSpotPublicData, sub.instrument, Trade)
                                })
                                .collect(),
                            channels.trade.entry(exchange).or_default().tx.clone(),
                        ));
                    }
                    (ExchangeId::OkxSpot, StreamKind::Trades) => {
                        unimplemented!()
                    }
                    (ExchangeId::OkxSpot, StreamKind::L2) => {
                        unimplemented!()
                    }
                    (ExchangeId::OkxSpot, StreamKind::AggTrades) => {
                        unimplemented!()
                    }
                    /*----- */
                    // KuCoin Spot
                    /*----- */
                    (ExchangeId::KuCoinSpot, StreamKind::Snapshot) => {
                        tokio::spawn(consume::<KuCoinSpotPublicData, OrderBookSnapshot>(
                            subs.into_iter()
                                .map(|sub| {
                                    Subscription::new(
                                        KuCoinSpotPublicData,
                                        sub.instrument,
                                        OrderBookSnapshot,
                                    )
                                })
                                .collect(),
                            channels.snapshots.entry(exchange).or_default().tx.clone(),
                        ));
                    }
                    (ExchangeId::KuCoinSpot, StreamKind::Trade) => {
                        tokio::spawn(consume::<KuCoinSpotPublicData, Trade>(
                            subs.into_iter()
                                .map(|sub| {
                                    Subscription::new(KuCoinSpotPublicData, sub.instrument, Trade)
                                })
                                .collect(),
                            channels.trade.entry(exchange).or_default().tx.clone(),
                        ));
                    }
                    (ExchangeId::KuCoinSpot, StreamKind::Trades) => {
                        unimplemented!()
                    }
                    (ExchangeId::KuCoinSpot, StreamKind::L2) => {
                        unimplemented!()
                    }
                    (ExchangeId::KuCoinSpot, StreamKind::AggTrades) => {
                        unimplemented!()
                    }
                    /*----- */
                    // Exmo Spot
                    /*----- */
                    (ExchangeId::ExmoSpot, StreamKind::Snapshot) => {
                        tokio::spawn(consume::<ExmoSpotPublicData, OrderBookSnapshot>(
                            subs.into_iter()
                                .map(|sub| {
                                    Subscription::new(
                                        ExmoSpotPublicData,
                                        sub.instrument,
                                        OrderBookSnapshot,
                                    )
                                })
                                .collect(),
                            channels.snapshots.entry(exchange).or_default().tx.clone(),
                        ));
                    }
                    (ExchangeId::ExmoSpot, StreamKind::Trades) => {
                        tokio::spawn(consume::<ExmoSpotPublicData, Trades>(
                            subs.into_iter()
                                .map(|sub| {
                                    Subscription::new(ExmoSpotPublicData, sub.instrument, Trades)
                                })
                                .collect(),
                            channels.trades.entry(exchange).or_default().tx.clone(),
                        ));
                    }
                    (ExchangeId::ExmoSpot, StreamKind::Trade) => {
                    }
                    (ExchangeId::ExmoSpot, StreamKind::L2) => {
                        unimplemented!()
                    }
                    (ExchangeId::ExmoSpot, StreamKind::AggTrades) => {
                        unimplemented!()
                    }
                    /*----- */
                    // Ascendex Spot
                    /*----- */
                    (ExchangeId::AscendExSpot, StreamKind::Snapshot) => {
                        // tokio::spawn(consume::<ExmoSpotPublicData, OrderBookSnapshot>(
                        //     subs.into_iter()
                        //         .map(|sub| {
                        //             Subscription::new(
                        //                 ExmoSpotPublicData,
                        //                 sub.instrument,
                        //                 OrderBookSnapshot,
                        //             )
                        //         })
                        //         .collect(),
                        //     channels.snapshots.entry(exchange).or_default().tx.clone(),
                        // ));
                    }
                    (ExchangeId::AscendExSpot, StreamKind::Trades) => {
                        // tokio::spawn(consume::<ExmoSpotPublicData, Trades>(
                        //     subs.into_iter()
                        //         .map(|sub| {
                        //             Subscription::new(ExmoSpotPublicData, sub.instrument, Trades)
                        //         })
                        //         .collect(),
                        //     channels.trades.entry(exchange).or_default().tx.clone(),
                        // ));
                    }
                    (ExchangeId::AscendExSpot, StreamKind::Trade) => {
                    }
                    (ExchangeId::AscendExSpot, StreamKind::L2) => {
                        unimplemented!()
                    }
                    (ExchangeId::AscendExSpot, StreamKind::AggTrades) => {
                        unimplemented!()
                    }
                };
            }
        }

        Ok(Self {
            trade: channels
                .trade
                .into_iter()
                .map(|(exchange, channel)| (exchange, UnboundedReceiverStream::new(channel.rx)))
                .collect(),
            trades: channels
                .trades
                .into_iter()
                .map(|(exchange, channel)| (exchange, UnboundedReceiverStream::new(channel.rx)))
                .collect(),
            l2s: channels
                .l2s
                .into_iter()
                .map(|(exchange, channel)| (exchange, UnboundedReceiverStream::new(channel.rx)))
                .collect(),
            snapshots: channels
                .snapshots
                .into_iter()
                .map(|(exchange, channel)| (exchange, UnboundedReceiverStream::new(channel.rx)))
                .collect(),
        })
    }

    pub fn select_trades(
        &mut self,
        exchange: ExchangeId,
    ) -> Option<UnboundedReceiverStream<MarketEvent<EventTrade>>> {
        self.trade.remove(&exchange)
    }

    pub fn select_all_trades(
        &mut self,
    ) -> SelectAll<UnboundedReceiverStream<MarketEvent<EventTrade>>> {
        select_all(std::mem::take(&mut self.trade).into_values())
    }

    pub fn select_l2s(
        &mut self,
        exchange: ExchangeId,
    ) -> Option<UnboundedReceiverStream<MarketEvent<EventOrderBook>>> {
        self.l2s.remove(&exchange)
    }

    pub fn select_all_l2s(
        &mut self,
    ) -> SelectAll<UnboundedReceiverStream<MarketEvent<EventOrderBook>>> {
        select_all(std::mem::take(&mut self.l2s).into_values())
    }

    pub fn select_all<Output>(self) -> impl Stream<Item = Output>
    where
        Output: 'static,
        MarketEvent<EventTrade>: Into<Output>,
        MarketEvent<Vec<EventTrade>>: Into<Output>,
        MarketEvent<EventOrderBook>: Into<Output>,
        MarketEvent<EventOrderBookSnapshot>: Into<Output>,
    {
        let Self {
            trade,
            trades,
            l2s,
            snapshots,
        } = self;
        let trade = trade
            .into_values()
            .map(|stream| stream.map(MarketEvent::into).boxed());

        let trades = trades
            .into_values()
            .map(|stream| stream.map(MarketEvent::into).boxed());

        let l2s = l2s
            .into_values()
            .map(|stream| stream.map(MarketEvent::into).boxed());

        let snapshots = snapshots
            .into_values()
            .map(|stream| stream.map(MarketEvent::into).boxed());

        let all = trade.chain(l2s).chain(snapshots).chain(trades);

        select_all(all)
    }
}

/*----- */
// Dynamic stream channels
/*----- */
#[derive(Default)]
struct Channels {
    l2s: HashMap<ExchangeId, ExchangeChannel<MarketEvent<EventOrderBook>>>,
    trade: HashMap<ExchangeId, ExchangeChannel<MarketEvent<EventTrade>>>,
    trades: HashMap<ExchangeId, ExchangeChannel<MarketEvent<Vec<EventTrade>>>>,
    snapshots: HashMap<ExchangeId, ExchangeChannel<MarketEvent<EventOrderBookSnapshot>>>,
}
