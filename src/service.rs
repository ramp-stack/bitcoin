//  use maverick_os::runtime::{Channel, Service, ServiceContext, async_trait, Callback, Error, BackgroundTask, ServiceList};
//  use maverick_os::hardware::{self, Cache};
//  use maverick_os::State;

//  use bdk_wallet::{Wallet, KeychainKind, ChangeSet, Update, LoadParams};
//  use bdk_wallet::descriptor::template::Bip86;
//  use bdk_wallet::bitcoin::bip32::Xpriv;
//  use bdk_wallet::bitcoin::{Amount, Network, Txid, FeeRate};
//  use bdk_wallet::{PersistedWallet, WalletPersister};
//  use bdk_wallet::chain::{Merge, ChainPosition, Anchor};
//  use bdk_esplora::esplora_client::Builder;
//  use bdk_esplora::EsploraExt;

//  use std::collections::BTreeMap;
//  use std::time::Duration;
//  use std::any::TypeId;
//  use std::pin::Pin;

//  use serde::{Serialize, Deserialize};
//  use serde_json::Value;

mod price;
pub use price::Price;

//  mod sync;
//  pub use sync::BDKSync;

//  #[derive(Debug)]
//  pub struct BDKError;
//  impl std::error::Error for BDKError {}
//  impl std::fmt::Display for BDKError {
//      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {write!(f, "{:?}", self)}
//  }

//  #[derive(Serialize, Deserialize, Default, Clone, Debug)]
//  pub struct WalletKey(Option<Xpriv>);

//  #[derive(Serialize, Deserialize, Default, Clone, Debug)]
//  pub struct Address(String);

//  #[derive(Serialize, Deserialize, Default, Debug, Clone)]
//  pub struct MemoryPersister(ChangeSet);
//  impl WalletPersister for MemoryPersister {
//      type Error = ();
//      fn initialize(persister: &mut Self) -> Result<ChangeSet, Self::Error> {Ok(persister.0.clone())}
//      fn persist(persister: &mut Self, changeset: &ChangeSet) -> Result<(), Self::Error> {persister.0.merge(changeset.clone()); Ok(())}
//  }
//  impl MemoryPersister {
//      pub async fn cache(mut self, cache: &mut Cache) {
//          let persister = cache.get::<MemoryPersister>().await;
//          self.0.merge(persister.0);
//          cache.set(&self).await;
//      }
//  }

//  fn price_service<'a>(ctx: &'a mut hardware::Context) -> Pin<Box<dyn Future<Output = Box<dyn Service>> + 'a >> {
//      Box::pin(async move {Box::new(price::PriceService::new(ctx).await) as Box<dyn Service>})
//  }

//  #[derive(Serialize, Deserialize)]
//  pub enum Request {
//      GetNewAddress
//  }

//  #[derive(Serialize, Deserialize)]
//  pub enum Response {
//      NewAddress(String)
//  }

//  pub struct BDKService;
//  #[async_trait]
//  impl Service for BDKService {
//      async fn new(ctx: &mut hardware::Context) -> Self {
//          //TODO: check cloud
//          let key = ctx.cache.get::<WalletKey>().await.0.unwrap_or(
//              Xpriv::new_master(Network::Bitcoin, &secp256k1::SecretKey::new(&mut secp256k1::rand::rng()).secret_bytes()).unwrap()
//          );
//          ctx.cache.set(&WalletKey(Some(key))).await;

//          let mut db = ctx.cache.get::<MemoryPersister>().await; 

//          let ext = Bip86(key, KeychainKind::External);
//          let int = Bip86(key, KeychainKind::Internal);
//          let network = Network::Bitcoin;
//          let wallet_opt = Wallet::load()
//              .descriptor(KeychainKind::External, Some(ext.clone()))
//              .descriptor(KeychainKind::Internal, Some(int.clone()))
//              .extract_keys()
//              .check_network(network)
//              .load_wallet(&mut db)
//              .expect("wallet");
//          match wallet_opt {
//              Some(wallet) => wallet,
//              None => {
//                  Wallet::create(ext, int)
//                  .network(network)
//                  .create_wallet(&mut db)
//                  .expect("wallet")
//              }
//          };
//          
//          BDKService
//      }

//      fn background_tasks(&self) -> Vec<Box<dyn BackgroundTask>> {vec![Box::new(BDKSync)]}

//      fn services(&self) -> ServiceList {
//          BTreeMap::from([(
//              TypeId::of::<BDKService>(), 
//              Box::new(price_service) as Box<dyn for<'a> FnOnce(&'a mut hardware::Context) -> Pin<Box<dyn Future<Output = Box<dyn Service>> + 'a>>>
//          )])
//      }

//      async fn run(&mut self, ctx: &mut ServiceContext, channel: &mut Channel) -> Result<Duration, Error> {
//          let mut persister = ctx.hardware.cache.get::<MemoryPersister>().await;
//          let mut wallet = PersistedWallet::load(&mut persister, LoadParams::new()).or(Err(BDKError))?.ok_or(BDKError)?;
//          while let Some(request) = channel.receive() {
//              match serde_json::from_str::<Request>(&request)? {
//                  Request::GetNewAddress => {
//                      let address = wallet.reveal_next_address(KeychainKind::External);
//                      channel.send(serde_json::to_string(&Response::NewAddress(address.address.to_string()))?);
//                  }
//              }
//          }
//          Ok(Duration::from_secs(1))
//      }

//      fn callback(&self) -> Box<Callback> {Box::new(|state: &mut State, response: String| {
//          match serde_json::from_str(&response).unwrap() {
//              Response::NewAddress(address) => state.set(&Address(address)),
//          }
//      })}
//  }
