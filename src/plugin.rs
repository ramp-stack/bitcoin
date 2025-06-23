use pelican_ui::{Context, Plugin};
use pelican_ui::runtime;

use crate::service::{BDKService, Request, Address, Price};

pub struct BDKPlugin(runtime::Context);
impl Plugin for BDKPlugin {
    fn new(ctx: &mut Context) -> Self {BDKPlugin(ctx.runtime.clone())}
}

impl BDKPlugin {
    pub fn request(&mut self, request: Request) {
        self.0.send::<BDKService>(&request)
    }

    pub fn balance(_ctx: &mut Context) -> f64 {
        0.0
    }

    pub fn price(ctx: &mut Context) -> f64 {
        ctx.state().get_or_default::<Price>().clone().0
    }

    pub fn address(ctx: &mut Context) -> String {
        let address = ctx.state().get_or_default::<Address>().clone().0;
        ctx.runtime.send::<BDKService>(&Request::GetNewAddress);
        address
    }
}