use maverick_os::runtime::{Channel, Service, ServiceContext, async_trait, Callback, Error, BackgroundTask, ServiceList};
use maverick_os::hardware::{self, Cache};
use maverick_os::State;

use bdk_wallet::{Wallet, KeychainKind, ChangeSet, Update, LoadParams};
use bdk_wallet::descriptor::template::Bip86;
use bdk_wallet::bitcoin::bip32::Xpriv;
use bdk_wallet::bitcoin::{Amount, Network, Txid, FeeRate};
use bdk_wallet::{PersistedWallet, WalletPersister};
use bdk_wallet::chain::{Merge, ChainPosition, Anchor};
use bdk_esplora::esplora_client::Builder;
use bdk_esplora::EsploraExt;

use std::collections::BTreeMap;
use std::time::Duration;
use std::any::TypeId;
use std::pin::Pin;

use serde::{Serialize, Deserialize};
use serde_json::Value;

use super::MemoryPersister;

#[derive(Debug)]
pub struct BDKError;
impl std::error::Error for BDKError {}
impl std::fmt::Display for BDKError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {write!(f, "{:?}", self)}
}

pub struct BDKSync;
#[async_trait]
impl BackgroundTask for BDKSync {
    async fn run(&mut self, ctx: &mut hardware::Context) -> Result<Duration, Error> {
        let mut persister = ctx.cache.get::<MemoryPersister>().await;
        let mut wallet = PersistedWallet::load(&mut persister, LoadParams::new()).or(Err(BDKError))?.ok_or(BDKError)?;

        let scan_request = wallet.start_full_scan().build();

        let builder = Builder::new("https://blockstream.info/api");
        let blocking_client = builder.build_blocking();
        let res = blocking_client.full_scan::<bdk_wallet::KeychainKind, _>(scan_request, 10, 1)?;
        wallet.apply_update(Update::from(res))?;

        wallet.persist(&mut persister).or(Err(BDKError))?;
        persister.cache(&mut ctx.cache).await;
        Ok(Duration::from_secs(5))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub datetime: Option<DateTime<Utc>>,
    pub txid: Txid,
    pub received: bool,
    pub address: String
    pub amount: u64,
    pub price: f64,
    pub fee: u64,
}

impl Transaction {
    pub async fn new(ctx: &mut ServiceContext, wtx: WalletTx) -> Result<Self, Error> {
        let tx = wtx.tx_node.tx;
        let datetime = match &wtx.chain_position {
            ChainPosition::Confirmed { anchor, .. } => {
                Some(Utc.timestamp_opt(anchor.confirmation_time as i64, 0).unwrap())
            }
            _ => None
        };

        let btc_price = ctx.get::<PriceService>().from_timestamp(datetime.unwrap_or(Utc::now()).timestamp()).await?;

        todo!()

      //Transaction {
      //    datetime,
      //    txid: wtx.tx_node.txid,

      //}

    }
}

//  for canonical_tx in wallet.transactions_sort_by(|tx1, tx2| tx2.chain_position.cmp(&tx1.chain_position)) {
//              let tx_node = &canonical_tx.tx_node;
//              let tx = &tx_node.tx;

//              let datetime) = match &wtx.chain_position {
//                  ChainPosition::Confirmed { anchor, .. } => {
//                      let unix_timestamp = get_block_time(anchor.anchor_block().hash.to_string()).await.unwrap_or(0);
//                      Some(Utc.timestamp_opt(unix_timestamp as i64, 0).unwrap())
//                  }
//                  _ => None
//              };
//              let btc_price = get_btc_price_at(utc.format("%Y-%m-%d %H:%M:%S").to_string()).await.unwrap_or(0.0);
//              
//              let received: u64 = tx.output.iter().filter(|out| wallet.is_mine(out.script_pubkey.clone())).map(|out| out.value.to_sat()).sum();
//              let input_sum: u64 = tx.input.iter().filter_map(|input| wallet.get_utxo(input.previous_output)).map(|utxo| utxo.txout.value.to_sat()).sum();
//              let sent = input_sum.saturating_sub(received);
//              let fee = (input_sum > 0).then(|| input_sum.saturating_sub(tx.output.iter().map(|o| o.value.to_sat()).sum())); // this has to be wrong...
//              let is_received = received > sent;

//              let address = match is_received {
//                  true => { 
//                      tx.output.iter().find_map(|out| {
//                          wallet.is_mine(out.script_pubkey.clone())
//                              .then(|| Address::from_script(&out.script_pubkey, wallet.network()).ok()).flatten()
//                      })                    
//                  },
//                  false => {
//                      tx.input.iter().find_map(|input| {
//                          wallet.get_utxo(input.previous_output).and_then(|utxo| {
//                              Address::from_script(&utxo.txout.script_pubkey, wallet.network()).ok()
//                          })
//                      })
//                  }
//              };

//              transactions.push(BDKTransaction {
//                  txid: tx_node.txid,
//                  datetime,
//                  is_received,
//                  amount: Amount::from_sat(received.max(sent)),
//                  price: btc_price,
//                  fee: fee.map(Amount::from_sat),
//                  address: address.map(|a| a.to_string()),
//              });
//          }

