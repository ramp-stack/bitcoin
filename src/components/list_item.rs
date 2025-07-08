use pelican_ui::Context;

use pelican_ui_std::{
    ListItem,
    Timestamp,
    NavigateEvent,
    AvatarContent,
    AvatarIconStyle,
};

use crate::{format_usd, format_nano_btc};

pub struct ListItemBitcoin;

impl ListItemBitcoin {
    pub fn bitcoin(ctx: &mut Context, is_received: bool, btc: f64, price: f64, date: Timestamp, on_click: impl FnMut(&mut Context) + 'static) -> ListItem {
        let title = if is_received { "Received bitcoin" } else { "Sent bitcoin" };
        let usd = &format_usd(btc * price);
        ListItem::new(ctx, true, title, None, Some(&date.friendly()), None, Some(usd), Some("Details"), None, None, None, on_click)
    }

    pub fn bitcoin_sending(ctx: &mut Context, btc: f64, price: f64, on_click: impl FnMut(&mut Context) + 'static) -> ListItem {
        let color = ctx.theme.colors.status.warning;
        let usd = &format_usd(btc * price);
        let btc = &format_nano_btc(btc);
        ListItem::new(ctx, true, "Sending bitcoin", Some(("warning", color)), Some("unconfirmed"), None, Some(usd), Some(btc), None, None, None, on_click)
    }

    pub fn wallet(ctx: &mut Context, is_spending: bool, btc: f64, price: f64, name: String) -> ListItem {
        let usd = &format_usd(btc * price);
        let btc = &format_nano_btc(btc);
        let subtitle = if is_spending {"Spending Wallet"} else {"Savings Wallet"};
        let icon = AvatarContent::Icon("wallet", AvatarIconStyle::Brand);
        ListItem::new(ctx, true, &name, None, Some(subtitle), None, Some(usd), Some(btc), None, Some(icon), None, |ctx: &mut Context| ctx.trigger_event(NavigateEvent(0)))
    }
}
