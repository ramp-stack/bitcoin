use pelican_ui::Context;
use pelican_ui::air::OrangeName;
use pelican_ui_std::AppPage;
use crate::pages::BitcoinHome;

pub struct IconButtonBitcoin;
impl IconButtonBitcoin {
    pub fn new(ctx: &mut Context) -> (&'static str, Box<dyn FnMut(&mut Context) -> Box<dyn AppPage>>) {
        // let label = if is_blocked { "unblock" } else { "block" };
        let closure = Box::new(move |ctx: &mut Context| {
            // let application_page = match is_blocked { 
            //     true => UnblockUser::new(ctx, &orange_name, account_return.lock().unwrap().take().unwrap()),
            //     false => BlockUser::new(ctx, &orange_name, account_return.lock().unwrap().take().unwrap())
            // };

            // ctx.trigger_event(NavigateEvent::new(application_page));
            Box::new(BitcoinHome::new(ctx)) as Box<dyn AppPage>
        });

        ("bitcoin", closure)
    }
}