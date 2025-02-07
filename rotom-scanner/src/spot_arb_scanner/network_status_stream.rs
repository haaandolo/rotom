use rotom_data::{
    exchange::PublicHttpConnector, model::network_info::NetworkSpecs,
    shared::subscription_models::Instrument, streams::builder::single::ExchangeChannel,
};
use tokio::{sync::mpsc, time::sleep, time::Duration};
use tracing::warn;

#[derive(Debug, Default)]
pub struct NetworkStatusStream(pub ExchangeChannel<NetworkSpecs>);

impl NetworkStatusStream {
    pub fn new() -> Self {
        Self(ExchangeChannel::default())
    }

    pub fn add_exchange<Exchange>(self, instruments: Vec<Instrument>) -> Self
    where
        Exchange: PublicHttpConnector + 'static,
        Exchange::NetworkInfo: Into<NetworkSpecs>,
    {
        let network_status_tx = self.0.tx.clone();
        tokio::spawn(async move {
            send_network_status_snapshots::<Exchange>(instruments, network_status_tx).await
        });
        self
    }

    pub fn build(mut self) -> mpsc::UnboundedReceiver<NetworkSpecs> {
        let (network_tx, network_rx) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            while let Some(network_status_data) = self.0.rx.recv().await {
                let _ = network_tx.send(network_status_data);
            }
        });
        network_rx
    }
}

async fn send_network_status_snapshots<Exchange>(
    instruments: Vec<Instrument>,
    network_status_tx: mpsc::UnboundedSender<NetworkSpecs>,
) where
    Exchange: PublicHttpConnector,
    Exchange::NetworkInfo: Into<NetworkSpecs>,
{
    loop {
        let network_specs_result = Exchange::get_network_info(instruments.clone()).await;
        match network_specs_result {
            Ok(network_specs) => {
                let _ = network_status_tx.send(network_specs.into());
            }
            Err(error) => {
                warn!(
                    exchange = %Exchange::ID,
                    error = %error
                )
            }
        }

        sleep(Duration::from_secs(60)).await;
    }
}
