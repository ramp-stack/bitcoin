use pelican_ui::runtime::Error;
use pelican_ui::hardware::Cache;
// use pelican_ui::State;

use bdk_wallet::{KeychainKind, ChangeSet, Update, LoadParams};
use bdk_wallet::descriptor::template::Bip86;
use bdk_wallet::bitcoin::bip32::Xpriv;
use bdk_wallet::bitcoin::{Network, Txid, Address};
use bdk_wallet::{PersistedWallet, WalletPersister};
use bdk_wallet::chain::{Merge, ChainPosition};
use bdk_esplora::esplora_client::Builder;
use bdk_esplora::EsploraExt;
// use bdk_wallet::bitcoin;

use serde::{Serialize, Deserialize};
use chrono::{DateTime, TimeZone, Utc};

use crate::service::price::PriceService;

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
struct MemoryPersister(ChangeSet);
impl WalletPersister for MemoryPersister {
    type Error = Error;
    fn initialize(persister: &mut Self) -> Result<ChangeSet, Self::Error> {Ok(persister.0.clone())}
    fn persist(persister: &mut Self, changeset: &ChangeSet) -> Result<(), Self::Error> {persister.0.merge(changeset.clone()); Ok(())}
}

pub struct Wallet(PersistedWallet<MemoryPersister>, MemoryPersister);
impl Wallet {
    pub async fn new(cache: &mut Cache) -> Result<Self, Error> {
        let mut persister = cache.get("MemoryPersister").await;
        let wallet = match PersistedWallet::load(&mut persister, LoadParams::new())? {
            Some(wallet) => wallet,
            None => {
                //TODO: check cloud
                let key = if let Some(k) = cache.get::<Option<Xpriv>>("WalletKey").await {k} else {
                    let key = Xpriv::new_master(Network::Bitcoin, &secp256k1::SecretKey::new(&mut secp256k1::rand::rng()).secret_bytes()).unwrap();
                    cache.set("WalletKey", &Some(key)).await;
                    key
                };
                bdk_wallet::Wallet::create(Bip86(key, KeychainKind::External), Bip86(key, KeychainKind::Internal))
                    .network(Network::Bitcoin).create_wallet(&mut persister)?
            }
        };
        let mut wallet = Wallet(wallet, persister);
        wallet.cache(cache).await?;
        Ok(wallet)
    }

    pub async fn cache(&mut self, cache: &mut Cache) -> Result<(), Error> {
        self.0.persist(&mut self.1)?;
        let persister: MemoryPersister = cache.get("MemoryPersister").await;
        self.1.0.merge(persister.0);
        cache.set("MemoryPersister", &self.1.clone()).await;
        Ok(())
    }

    pub async fn scan(&mut self) -> Result<(), Error> {
        let scan_request = self.0.start_full_scan().inspect(|k, i, s| println!("scaning: {:?}, {}, {:?}", k, i, s)).build();

        let builder = Builder::new("https://blockstream.info/api");
        let blocking_client = builder.build_blocking();
        let res = blocking_client.full_scan(scan_request, 10, 1)?;
        self.0.apply_update(Update::from(res))?;
        Ok(())
    }

    pub async fn transactions(&mut self, cache: &mut Cache) -> Result<Vec<Transaction>, Error> {
        let mut transactions = Vec::new();
        let wtxs = self.0.transactions_sort_by(|tx1, tx2| tx2.chain_position.cmp(&tx1.chain_position)).into_iter().map(|tx| {
            let datetime = match &tx.chain_position {
                ChainPosition::Confirmed { anchor, .. } => {
                    Some(Utc.timestamp_opt(anchor.confirmation_time as i64, 0).unwrap())
                }
                _ => None
            };
            (tx.tx_node.txid, datetime, tx.tx_node.tx)
        }).collect::<Vec<_>>();
        for (txid, datetime, tx) in wtxs {
            //transactions.push(Transaction::new(ctx, &mut self.0, txid, datetime, bitcoin::Transaction::clone(&tx)).await?);
            let btc_price = PriceService::from_timestamp(cache, datetime.unwrap_or(Utc::now())).await?;

            let total_out: u64 = tx.output.iter().map(|out| out.value.to_sat()).sum();
            let total_in: u64 = tx.input.iter().map(|utxo| self.0.get_utxo(utxo.previous_output).unwrap().txout.value.to_sat()).sum();
            let sent: u64 = tx.input.iter().filter(|txin| self.0.is_mine(txin.script_sig.clone())).map(|txin| self.0.get_utxo(txin.previous_output).unwrap().txout.value.to_sat()).sum();
            let received: u64 = tx.output.iter().filter(|out| self.0.is_mine(out.script_pubkey.clone())).map(|out| out.value.to_sat()).sum();
            let fee = total_in - total_out;
            let is_received = received > sent;

            let address = match is_received {
                true => { 
                    tx.output.iter().find_map(|out| {
                        self.0.is_mine(out.script_pubkey.clone()).then(|| Address::from_script(&out.script_pubkey, self.0.network()).ok()).flatten()
                    }).unwrap()
                },
                false => {
                    tx.input.iter().find_map(|input| {
                        self.0.get_utxo(input.previous_output).and_then(|utxo| {
                            Address::from_script(&utxo.txout.script_pubkey, self.0.network()).ok()
                        })
                    }).unwrap()
                }
            };

            transactions.push(Transaction{
                txid,
                datetime,
                received: is_received,
                address: address.to_string(),
                amount: if is_received {received} else {sent},
                price: btc_price,
                fee
            })
        }
        Ok(transactions)
    }

    pub fn get_new_address(&mut self) -> Address {
        self.0.reveal_next_address(KeychainKind::External).address
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
