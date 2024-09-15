use futures::{
    stream::{select_all, SelectAll},
    Stream, StreamExt,
};
use itertools::Itertools;
use std::{collections::HashMap, fmt::Debug};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing::error;
use vecmap::VecMap;

use super::single::ExchangeChannel;
use crate::{
    error::SocketError,
    event_models::{
        event_book::{EventOrderBook, OrderBookL2},
        event_trade::{AggTrades, EventTrade, Trades},
        market_event::MarketEvent,
    },
    exchange::{binance::BinanceSpot, poloniex::public::PoloniexSpot},
    shared::subscription_models::{ExchangeId, StreamKind, Subscription},
    streams::consumer::consume,
};

/*----- */
// Dynamic Streams
/*----- */
#[derive(Debug)]
pub struct DynamicStreams {
    pub trades: VecMap<ExchangeId, UnboundedReceiverStream<MarketEvent<EventTrade>>>,
    pub l2s: VecMap<ExchangeId, UnboundedReceiverStream<MarketEvent<EventOrderBook>>>,
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
                        tokio::spawn(consume::<BinanceSpot, OrderBookL2>(
                            subs.into_iter()
                                .map(|sub| {
                                    Subscription::new(BinanceSpot, sub.instrument, OrderBookL2)
                                })
                                .collect(),
                            channels.l2s.entry(exchange).or_default().tx.clone(),
                        ));
                    }
                    (ExchangeId::BinanceSpot, StreamKind::Trades) => {
                        tokio::spawn(consume::<BinanceSpot, Trades>(
                            subs.into_iter()
                                .map(|sub| Subscription::new(BinanceSpot, sub.instrument, Trades))
                                .collect(),
                            channels.trades.entry(exchange).or_default().tx.clone(),
                        ));
                    }
                    (ExchangeId::BinanceSpot, StreamKind::AggTrades) => {
                        tokio::spawn(consume::<BinanceSpot, AggTrades>(
                            subs.into_iter()
                                .map(|sub| {
                                    Subscription::new(BinanceSpot, sub.instrument, AggTrades)
                                })
                                .collect(),
                            channels.trades.entry(exchange).or_default().tx.clone(),
                        ));
                    }
                    /*----- */
                    // Poloniex Spot
                    /*----- */
                    (ExchangeId::PoloniexSpot, StreamKind::L2) => {
                        tokio::spawn(consume::<PoloniexSpot, OrderBookL2>(
                            subs.into_iter()
                                .map(|sub| {
                                    Subscription::new(PoloniexSpot, sub.instrument, OrderBookL2)
                                })
                                .collect(),
                            channels.l2s.entry(exchange).or_default().tx.clone(),
                        ));
                    }
                    (ExchangeId::PoloniexSpot, StreamKind::Trades) => {
                        tokio::spawn(consume::<PoloniexSpot, Trades>(
                            subs.into_iter()
                                .map(|sub| Subscription::new(PoloniexSpot, sub.instrument, Trades))
                                .collect(),
                            channels.trades.entry(exchange).or_default().tx.clone(),
                        ));
                    }
                    // Poloniex's does not separate regular and aggregated trades
                    (ExchangeId::PoloniexSpot, StreamKind::AggTrades) => {
                        error!(message = "Poloniex spot does not have a aggregated trades stream")
                    }
                };
            }
        }

        Ok(Self {
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
        })
    }

    pub fn select_trades(
        &mut self,
        exchange: ExchangeId,
    ) -> Option<UnboundedReceiverStream<MarketEvent<EventTrade>>> {
        self.trades.remove(&exchange)
    }

    pub fn select_all_trades(
        &mut self,
    ) -> SelectAll<UnboundedReceiverStream<MarketEvent<EventTrade>>> {
        select_all(std::mem::take(&mut self.trades).into_values())
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
        MarketEvent<EventOrderBook>: Into<Output>,
    {
        let Self { trades, l2s } = self;
        let trades = trades
            .into_values()
            .map(|stream| stream.map(MarketEvent::into).boxed());

        let l2s = l2s
            .into_values()
            .map(|stream| stream.map(MarketEvent::into).boxed());

        let all = trades.chain(l2s);

        select_all(all)
    }
}

/*----- */
// Dynamic stream channels
/*----- */
#[derive(Default)]
struct Channels {
    l2s: HashMap<ExchangeId, ExchangeChannel<MarketEvent<EventOrderBook>>>,
    trades: HashMap<ExchangeId, ExchangeChannel<MarketEvent<EventTrade>>>,
}
