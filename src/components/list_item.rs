use rust_on_rails::prelude::*;
use pelican_ui::prelude::*;
use crate::{format_usd, format_nano_btc};

pub trait ListItemBitcoin {
    fn bitcoin(ctx: &mut Context, is_received: bool, btc: f64, price: f64, date: Timestamp, on_click: impl FnMut(&mut Context) + 'static) -> Self;
    fn bitcoin_sending(ctx: &mut Context, btc: f64, price: f64, on_click: impl FnMut(&mut Context) + 'static) -> Self;
}

impl ListItemBitcoin for ListItem {
    /// Creates a list item for a completed Bitcoin transaction.
    /// Displays whether Bitcoin was received or sent, along with the transaction's USD value and date.
    fn bitcoin(ctx: &mut Context, is_received: bool, btc: f64, price: f64, date: Timestamp, on_click: impl FnMut(&mut Context) + 'static) -> Self {
        let title = if is_received { "Received bitcoin" } else { "Sent bitcoin" };
        let usd = &format_usd(btc * price);
        ListItem::new(ctx, true, title, None, Some(&date.friendly()), None, Some(usd), Some("Details"), None, None, None, on_click)
    }

    /// Creates a list item for a Bitcoin transaction still in the process of sending.
    /// Displays USD and BTC values, along with a warning flair to indicate the sending status.
    fn bitcoin_sending(ctx: &mut Context, btc: f64, price: f64, on_click: impl FnMut(&mut Context) + 'static) -> Self {
        let color = ctx.get::<PelicanUI>().theme.colors.status.warning;
        let usd = &format_usd(btc * price);
        let btc = &format_nano_btc(btc);
        ListItem::new(ctx, true, "Sending bitcoin", Some(("warning", color)), Some("unconfirmed"), None, Some(&usd), Some(&btc), None, None, None, on_click)
    }
}
