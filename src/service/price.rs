use pelican_ui::runtime::{Service, ThreadContext, async_trait, Error, Services};
use pelican_ui::hardware::{self, Cache};
use pelican_ui::State;

use std::collections::BTreeMap;
use std::time::Duration;

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use serde_json::Value;

#[derive(Debug)]
pub struct PriceError;
impl std::error::Error for PriceError {}
impl std::fmt::Display for PriceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {write!(f, "{:?}", self)}
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Price(pub f64);

pub struct PriceService();
#[async_trait]
impl Service for PriceService {
    type Send = f64;
    type Receive = ();
    async fn new(_hardware: &mut hardware::Context) -> Self {PriceService()}

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

    fn callback(state: &mut State, response: f64) {
        state.set(&Price(response));
    }
}
impl Services for PriceService {}
impl PriceService {
    pub async fn from_timestamp(hcache: &mut Cache, timestamp: DateTime<Utc>) -> Result<f64, Error> {
        let mut cache: BTreeMap<DateTime<Utc>, f64> = hcache.get("PriceCache").await;
        Ok(match cache.get(&timestamp) {
            Some(price) => *price,
            None => {
                let date = timestamp.format("%Y-%m-%d");
                let url = format!("https://api.coinbase.com/v2/prices/BTC-USD/spot?date={}", date);
                let json: Value = reqwest::get(&url).await?.json().await?;
                let price_str = json.get("data").ok_or(PriceError)?
                                    .get("amount").ok_or(PriceError)?
                                    .as_str().ok_or(PriceError)?;
                let price = price_str.parse()?;
                cache.insert(timestamp, price);
                hcache.set("PriceCache", cache).await;
                price
            }
        })
    }
}
