use pelican_ui::runtime::{Service, Services, ThreadContext, async_trait, Error, ServiceList, BackgroundList};
use pelican_ui_std::InternetConnection;
use pelican_ui::hardware::{self};
use pelican_ui::State;

// use bdk_wallet::{KeychainKind, ChangeSet, Update, LoadParams};
// use bdk_wallet::descriptor::template::Bip86;
// use bdk_wallet::bitcoin::bip32::Xpriv;
// use bdk_wallet::bitcoin::{Amount, Network, Txid, FeeRate};
// use bdk_wallet::{PersistedWallet, WalletPersister};
// use bdk_wallet::chain::{Merge, ChainPosition, Anchor};
// use bdk_esplora::esplora_client::Builder;
// use bdk_esplora::EsploraExt;

use std::time::Duration;

use serde::{Serialize, Deserialize};

pub mod price;
pub use price::Price;

mod sync;
use sync::BDKSync;

use crate::wallet::Wallet;

#[derive(Debug)]
pub struct BDKError;
impl std::error::Error for BDKError {}
impl std::fmt::Display for BDKError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {write!(f, "{:?}", self)}
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Address(pub String);

#[derive(Serialize, Deserialize)]
pub enum Request {
    GetNewAddress
}

#[derive(Serialize, Deserialize)]
pub enum Response {
    NewAddress(String)
}

pub struct BDKService(bool);
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

    async fn new(_ctx: &mut hardware::Context) -> Self {
        BDKService(false)
    }

    fn background_tasks() -> BackgroundList {
        let mut background = BackgroundList::default();
        background.insert::<BDKSync>();
        background
    }

    async fn run(&mut self, ctx: &mut ThreadContext<Self::Send, Self::Receive>) -> Result<Option<Duration>, Error> {
        let mut wallet = Wallet::new(&mut ctx.hardware.cache).await?;
        if !self.0 {
            println!("Got Address!");
            self.0 = true;
            let address = wallet.get_new_address();
            ctx.callback(Response::NewAddress(address.to_string()));
        }
        while let Some((id, request)) = ctx.get_request() {
            match request {
                Request::GetNewAddress => {
                    let address = wallet.get_new_address();
                    ctx.respond(id, Response::NewAddress(address.to_string()));
                }
            }
        }
        wallet.cache(&mut ctx.hardware.cache).await?;
        Ok(Some(Duration::ZERO))
    }

    fn callback(state: &mut State, response: Self::Send) {
        match response {
            Response::NewAddress(address) => state.set(Address(address)),
        }
    }
}
