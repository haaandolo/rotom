use crate::{error::SocketError, shared::subscription_models::Instrument};

use super::requests::ticker_info::PoloniexSpotTickerInfo;

pub const HTTP_TICKER_INFO_URL_POLONIEX_SPOT: &str = "https://api.poloniex.com/markets/";

/*----- */
// Binance Public Data
/*----- */
#[derive(Debug)]
pub struct PoloniexPublicData;

impl PoloniexPublicData {
    // This function returns a Vec<PoloniexSpotTickerInfo> but the function only
    // takes in a single instrument, we should only get a Vec of len == 1. So
    // here we can index by 0 i.e. info[0] to get the res.
    pub async fn get_ticker_info(
        instrument: &Instrument,
    ) -> Result<Vec<PoloniexSpotTickerInfo>, SocketError> {
        let ticker_info_url = format!(
            "{}{}_{}",
            HTTP_TICKER_INFO_URL_POLONIEX_SPOT,
            instrument.base.to_uppercase(),
            instrument.quote.to_uppercase()
        );

        reqwest::get(ticker_info_url)
            .await
            .map_err(SocketError::Http)?
            .json::<Vec<PoloniexSpotTickerInfo>>()
            .await
            .map_err(SocketError::Http)
    }
}
