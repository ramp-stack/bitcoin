use pelican_ui::runtime::{Channel, Service, async_trait, Callback, Error, BackgroundTask, ServiceList};
use pelican_ui::hardware::{self, Cache};
use pelican_ui::State;

use bdk_wallet::{KeychainKind, ChangeSet, Update, LoadParams, WalletTx};
use bdk_wallet::descriptor::template::Bip86;
use bdk_wallet::bitcoin::bip32::Xpriv;
use bdk_wallet::bitcoin::{Amount, Network, Txid, FeeRate, Address};
use bdk_wallet::{PersistedWallet, WalletPersister};
use bdk_wallet::chain::{Merge, ChainPosition, Anchor};
use bdk_esplora::esplora_client::Builder;
use bdk_esplora::EsploraExt;
use bdk_wallet::bitcoin;

use std::collections::BTreeMap;
use std::time::Duration;
use std::any::TypeId;
use std::pin::Pin;

use serde::{Serialize, Deserialize};
use chrono::{DateTime, TimeZone, Utc};
use serde_json::Value;

use super::price::PriceService;

use crate::wallet::Wallet;

pub struct BDKSync(Wallet);
#[async_trait]
impl BackgroundTask for BDKSync {
    async fn new(ctx: &mut hardware::Context) -> Self {
        BDKSync(Wallet::new(&mut ctx.cache).await.unwrap())
    }

    async fn run(&mut self, ctx: &mut hardware::Context) -> Result<Option<Duration>, Error> {
        self.0.scan().await?;

        let transactions = self.0.transactions(&mut ctx.cache).await?;
        ctx.cache.set("Transactions", transactions).await;

        self.0.cache(&mut ctx.cache).await?;
        Ok(Some(Duration::from_secs(5)))
    }
}
