use pelican_ui::Context;
use pelican_ui_std::AppPage;
use crate::pages::BitcoinHome;

type BitcoinButton = (&'static str, Box<dyn FnMut(&mut Context) -> Box<dyn AppPage>>);

pub struct IconButtonBitcoin;
impl IconButtonBitcoin {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(_ctx: &mut Context) -> BitcoinButton {
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