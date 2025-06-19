use pelican_ui::runtime::{Service, ThreadContext, async_trait, Error, Services};
use pelican_ui::hardware;
use pelican_ui::State;

use std::time::Duration;

use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Debug)]
pub struct PriceError;
impl std::error::Error for PriceError {}
impl std::fmt::Display for PriceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {write!(f, "{:?}", self)}
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Price(pub f32);

pub struct PriceService;
#[async_trait]
impl Service for PriceService {
    type Send = f32;
    type Receive = ();
    async fn new(_hardware: &mut hardware::Context) -> Self {PriceService}

    async fn run(&mut self, ctx: &mut ThreadContext<Self::Send, Self::Receive>) -> Result<Option<Duration>, Error> {
        let url = "https://api.coinbase.com/v2/prices/spot?currency=USD";
        let body = reqwest::get(url).await?.text().await?;
        let json: Value = serde_json::from_str(&body)?;
        let price_str = json.get("data").ok_or(PriceError)?
                            .get("amount").ok_or(PriceError)?
                            .as_str().ok_or(PriceError)?;
        ctx.callback(price_str.parse()?);
        Ok(Some(Duration::from_secs(10)))
    }

    fn callback(state: &mut State, response: f32) {
        state.set(&Price(response));
    }
}
impl Services for PriceService {}
impl PriceService {
    async fn from_timestamp(timestamp: i64) -> Result<f64, Error> {
        let from = timestamp - 300;
        let to = timestamp + 300;
        let url = format!("https://api.coingecko.com/api/v3/coins/bitcoin/market_chart/range?vs_currency=usd&from={from}&to={to}");
        let val: Value = reqwest::get(&url).await?.json().await?;
        let prices = val.get("prices").ok_or(PriceError)?.as_array().ok_or(PriceError)?;

        let closest_price = prices
            .iter()
            .filter_map(|entry| {
                let arr = entry.as_array()?;
                let ts = arr[0].as_f64()? as i64 / 1000;
                let price = arr[1].as_f64()?;
                Some((ts, price))
            })
            .min_by_key(|(ts, _)| (ts - timestamp).abs())
            .map(|(_, price)| price)
            .ok_or(PriceError)?;

        Ok(closest_price)
    }
}
