use pelican_ui::runtime::{Channel, Service, async_trait, Callback, Error, BackgroundTask, ServiceList};
use pelican_ui::hardware::{self, Cache};
use pelican_ui::State;

use bdk_wallet::{Wallet, KeychainKind, ChangeSet, Update, LoadParams, WalletTx};
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

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct MemoryPersister(ChangeSet);
impl WalletPersister for MemoryPersister {
    type Error = ();
    fn initialize(persister: &mut Self) -> Result<ChangeSet, Self::Error> {Ok(persister.0.clone())}
    fn persist(persister: &mut Self, changeset: &ChangeSet) -> Result<(), Self::Error> {persister.0.merge(changeset.clone()); Ok(())}
}
impl MemoryPersister {
    pub async fn from_cache(cache: &mut Cache) -> Self {
        cache.get("MemoryPersister").await
    }
    pub async fn cache(mut self, cache: &mut Cache) {
        let persister: MemoryPersister = cache.get("MemoryPersister").await;
        self.0.merge(persister.0);
        cache.set("MemoryPersister", self).await;
    }
}

#[derive(Debug)]
pub struct BDKError;
impl std::error::Error for BDKError {}
impl std::fmt::Display for BDKError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {write!(f, "{:?}", self)}
}

pub struct BDKSync(PersistedWallet<MemoryPersister>);
#[async_trait]
impl BackgroundTask for BDKSync {
    async fn new(ctx: &mut hardware::Context) -> Self {
        //TODO: check cloud
        let key = ctx.cache.get::<Option<Xpriv>>("WalletKey").await.unwrap_or(
            Xpriv::new_master(Network::Bitcoin, &secp256k1::SecretKey::new(&mut secp256k1::rand::rng()).secret_bytes()).unwrap()
        );
        ctx.cache.set("WalletKey", Some(key)).await;

        let mut db = ctx.cache.get("MemoryPersister").await; 

        let ext = Bip86(key, KeychainKind::External);
        let int = Bip86(key, KeychainKind::Internal);
        let network = Network::Bitcoin;
        let wallet_opt = Wallet::load()
            .descriptor(KeychainKind::External, Some(ext.clone()))
            .descriptor(KeychainKind::Internal, Some(int.clone()))
            .extract_keys()
            .check_network(network)
            .load_wallet(&mut db)
            .expect("wallet");
        let wallet = match wallet_opt {
            Some(wallet) => wallet,
            None => {
                Wallet::create(ext, int)
                .network(network)
                .create_wallet(&mut db)
                .expect("wallet")
            }
        };
        
        BDKSync(wallet)
    }

    async fn run(&mut self, ctx: &mut hardware::Context) -> Result<Option<Duration>, Error> {
        let mut persister: MemoryPersister = ctx.cache.get("MemoryPersister").await;
        let mut wallet = PersistedWallet::load(&mut persister, LoadParams::new()).or(Err(BDKError))?.ok_or(BDKError)?;

        let scan_request = wallet.start_full_scan().build();

        let builder = Builder::new("https://blockstream.info/api");
        let blocking_client = builder.build_blocking();
        let res = blocking_client.full_scan(scan_request, 10, 1)?;
        wallet.apply_update(Update::from(res))?;

        let mut transactions = Vec::new();
        let wtxs = wallet.transactions_sort_by(|tx1, tx2| tx2.chain_position.cmp(&tx1.chain_position)).into_iter().map(|tx| {
            let datetime = match &tx.chain_position {
                ChainPosition::Confirmed { anchor, .. } => {
                    Some(Utc.timestamp_opt(anchor.confirmation_time as i64, 0).unwrap())
                }
                _ => None
            };
            (tx.tx_node.txid, datetime, tx.tx_node.tx)
            }).collect::<Vec<_>>();
        for (txid, datetime, tx) in wtxs {
            transactions.push(Transaction::new(ctx, &mut wallet, txid, datetime, bitcoin::Transaction::clone(&tx)).await?)
        }
        ctx.cache.set("Transactions", transactions).await;


        wallet.persist(&mut persister).or(Err(BDKError))?;
        persister.cache(&mut ctx.cache).await;
        Ok(Some(Duration::from_secs(5)))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub txid: Txid,
    pub datetime: Option<DateTime<Utc>>,
    pub received: bool,
    pub address: String,
    pub amount: u64,
    pub price: f64,
    pub fee: u64,
}

impl Transaction {
    pub async fn new(hardware: &mut hardware::Context, wallet: &mut PersistedWallet<MemoryPersister>, txid: Txid, datetime: Option<DateTime<Utc>>, tx: bitcoin::Transaction) -> Result<Self, Error> {
        let btc_price = PriceService::from_timestamp(hardware, datetime.unwrap_or(Utc::now())).await?;

        let total_out: u64 = tx.output.iter().map(|out| out.value.to_sat()).sum();
        let total_in: u64 = tx.input.iter().map(|utxo| wallet.get_utxo(utxo.previous_output).unwrap().txout.value.to_sat()).sum();
        let sent: u64 = tx.input.iter().filter(|txin| wallet.is_mine(txin.script_sig.clone())).map(|txin| wallet.get_utxo(txin.previous_output).unwrap().txout.value.to_sat()).sum();
        let received: u64 = tx.output.iter().filter(|out| wallet.is_mine(out.script_pubkey.clone())).map(|out| out.value.to_sat()).sum();
        let fee = total_in - total_out;
        let is_received = received > sent;

        let address = match is_received {
            true => { 
                tx.output.iter().find_map(|out| {
                    wallet.is_mine(out.script_pubkey.clone()).then(|| Address::from_script(&out.script_pubkey, wallet.network()).ok()).flatten()
                }).unwrap()
            },
            false => {
                tx.input.iter().find_map(|input| {
                    wallet.get_utxo(input.previous_output).and_then(|utxo| {
                        Address::from_script(&utxo.txout.script_pubkey, wallet.network()).ok()
                    })
                }).unwrap()
            }
        };

        Ok(Transaction{
            txid,
            datetime,
            received: is_received,
            address: address.to_string(),
            amount: if is_received {received} else {sent},
            price: btc_price,
            fee
        })
    }
}
