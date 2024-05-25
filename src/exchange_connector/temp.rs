
// top level ws
impl<Exchange, Instrument, Kind, Transformer> MarketStream<Exchange, Instrument, Kind>
    for ExchangeWsStream<Transformer>
where
    Exchange: Connector + Send + Sync,
    Instrument: InstrumentData,
    Kind: SubKind + Send + Sync,
    Transformer: ExchangeTransformer<Exchange, Instrument::Id, Kind> + Send,
    Kind::Event: Send,
{
    async fn init(
        subscriptions: &[Subscription<Exchange, Instrument, Kind>],
    ) -> Result<Self, DataError>
    where
        Subscription<Exchange, Instrument, Kind>:
            Identifier<Exchange::Channel> + Identifier<Exchange::Market>,
    {
        // Connect & subscribe
        let (websocket, map) = Exchange::Subscriber::subscribe(subscriptions).await?;

        // Split WebSocket into WsStream & WsSink components
        let (ws_sink, ws_stream) = websocket.split();

        // Spawn task to distribute Transformer messages (eg/ custom pongs) to the exchange
        let (ws_sink_tx, ws_sink_rx) = mpsc::unbounded_channel();
        tokio::spawn(distribute_messages_to_exchange(
            Exchange::ID,
            ws_sink,
            ws_sink_rx,
        ));

        // Spawn optional task to distribute custom application-level pings to the exchange
        if let Some(ping_interval) = Exchange::ping_interval() {
            tokio::spawn(schedule_pings_to_exchange(
                Exchange::ID,
                ws_sink_tx.clone(),
                ping_interval,
            ));
        }

        // Construct Transformer associated with this Exchange and SubKind
        let transformer = Transformer::new(ws_sink_tx, map).await?;

        Ok(ExchangeWsStream::new(ws_stream, transformer))
    }
}


// ping scheduler
pub async fn schedule_pings_to_exchange(
    exchange: ExchangeId,
    ws_sink_tx: mpsc::UnboundedSender<WsMessage>,
    PingInterval { mut interval, ping }: PingInterval,
) {
    loop {
        // Wait for next scheduled ping
        interval.tick().await;

        // Construct exchange custom application-level ping payload
        let payload = ping();
        debug!(%exchange, %payload, "sending custom application-level ping to exchange");

        if ws_sink_tx.send(payload).is_err() {
            break;
        }
    }
}