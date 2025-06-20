use pelican_ui::runtime::{Channel, Service, Services, ThreadContext, async_trait, Error, BackgroundTask, ServiceList, BackgroundList};
use pelican_ui::hardware::{self, Cache};
use pelican_ui::State;

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

mod price;
pub use price::Price;

mod sync;
use sync::{MemoryPersister, BDKSync};

#[derive(Debug)]
pub struct BDKError;
impl std::error::Error for BDKError {}
impl std::fmt::Display for BDKError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {write!(f, "{:?}", self)}
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Address(String);

#[derive(Serialize, Deserialize)]
pub enum Request {
    GetNewAddress
}

#[derive(Serialize, Deserialize)]
pub enum Response {
    NewAddress(String)
}

pub struct BDKService;
impl Services for BDKService {
    fn services() -> ServiceList {
        let mut services = ServiceList::default();
        services.insert::<price::PriceService>();
        services
    }
}
#[async_trait]
impl Service for BDKService {
    type Send = Response;
    type Receive = Request;

    async fn new(ctx: &mut hardware::Context) -> Self {
        BDKService
    }

    fn background_tasks() -> BackgroundList {
        let mut background = BackgroundList::default();
        background.insert::<BDKSync>();
        background
    }

    async fn run(&mut self, ctx: &mut ThreadContext<Self::Send, Self::Receive>) -> Result<Option<Duration>, Error> {
        let mut persister = MemoryPersister::from_cache(&mut ctx.hardware.cache).await;
        if let Some(mut wallet) = PersistedWallet::load(&mut persister, LoadParams::new()).ok().and_then(|r| r) {
            while let Some((id, request)) = ctx.get_request() {
                match request {
                    Request::GetNewAddress => {
                        let address = wallet.reveal_next_address(KeychainKind::External);
                        ctx.respond(id, Response::NewAddress(address.address.to_string()));
                    }
                }
            }
        }
        Ok(Some(Duration::ZERO))
    }

    fn callback(state: &mut State, response: Self::Send) {
        match response {
            Response::NewAddress(address) => state.set(&Address(address)),
        }
    }
}
