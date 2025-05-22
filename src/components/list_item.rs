use rust_on_rails::prelude::*;
use pelican_ui::prelude::*;

pub trait ListItemBitcoin {
    fn bitcoin(ctx: &mut Context, is_received: bool, usd: &str, date: &str, on_click: impl FnMut(&mut Context) + 'static) -> Self;
    fn bitcoin_sending(ctx: &mut Context, usd: &str, btc: &str, date: &str, on_click: impl FnMut(&mut Context) + 'static) -> Self;
}

impl ListItemBitcoin for ListItem {
    /// Creates a list item for a completed Bitcoin transaction.
    /// Displays whether Bitcoin was received or sent, along with the transaction's USD value and date.
    fn bitcoin(ctx: &mut Context, is_received: bool, usd: &str, date: &str, on_click: impl FnMut(&mut Context) + 'static) -> Self {
        let title = if is_received { "Received bitcoin" } else { "Sent bitcoin" };
        ListItem::new(ctx, true, title, None, Some(date), None, Some(usd), Some("Details"), None, None, None, on_click)
    }

    /// Creates a list item for a Bitcoin transaction still in the process of sending.
    /// Displays USD and BTC values, along with a warning flair to indicate the sending status.
    fn bitcoin_sending(ctx: &mut Context, usd: &str, btc: &str, date: &str, on_click: impl FnMut(&mut Context) + 'static) -> Self {
        let color = ctx.get::<PelicanUI>().theme.colors.status.warning;
        ListItem::new(ctx, true, "Sending bitcoin", Some(("warning", color)), Some(date), None, Some(&usd), Some(&btc), None, None, None, on_click)
    }
}
