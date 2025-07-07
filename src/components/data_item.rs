use pelican_ui::Context;

use pelican_ui_std::{
    DataItem,
    Timestamp,
    Button,
};

use crate::{NANS, format_usd, format_nano_btc};

pub struct DataItemBitcoin;

impl DataItemBitcoin {
    pub fn confirm_address(ctx: &mut Context, address: &str, to_edit_address: impl FnMut(&mut Context) + 'static) -> DataItem {
        let edit_address = Button::secondary(ctx, Some("edit"), "Edit Address", None, to_edit_address, None);

        DataItem::new(ctx, None, "Confirm address", Some(address),
            Some("Bitcoin sent to the wrong address can never be recovered."),
            None, Some(vec![edit_address])
        )
    }

    pub fn confirm_amount(
        ctx: &mut Context, btc: f64, price: f64, fee: f64, is_priority: bool,
        to_edit_speed: impl FnMut(&mut Context) + 'static, 
        to_edit_amount: impl FnMut(&mut Context) + 'static
    ) -> DataItem {
        let speed = match is_priority {
            false => "Standard (~2 hours)",
            true => "Priority (~30 minutes)"
        };

        let amount_sent = &format_usd(btc*price);
        let bitcoin_sent = &format_nano_btc(btc*NANS);
        let total = &format_usd((btc*price)+fee);
        let network_fee = &format_usd(fee);
        let details: Vec<(&str, &str)> = vec![
            ("Amount sent", amount_sent),
            ("Bitcoin sent", bitcoin_sent),
            ("Send speed", speed),
            ("Network fee", network_fee),
            ("Total", total)
        ];

        let edit_speed = Button::secondary(ctx, Some("edit"), "Edit Speed", None, to_edit_speed, None);
        let edit_amount = Button::secondary(ctx, Some("edit"), "Edit Amount", None, to_edit_amount, None);
        DataItem::new(ctx, None, "Confirm amount", None, None, Some(details), Some(vec![edit_amount, edit_speed]))
    }

    pub fn received_tx(ctx: &mut Context, timestamp: Timestamp, btc: f64, price: f64, address: &str) -> DataItem {
        let (date, time) = (timestamp.date(), timestamp.time());
        let nano_btc = &format_nano_btc(btc * NANS);
        let usd = &format_usd(btc*price);
        let price = &format_usd(price);

        let details: Vec<(&str, &str)> = vec![
            ("Date", &date),
            ("Time", &time),
            ("Amount received", usd),
            ("Bitcoin received", nano_btc),
            ("Bitcoin price", price),
            ("Received at address", address),
        ];

        DataItem::new(ctx, None, "Transaction details", None, None, Some(details), None)
    }

    pub fn sent_tx(ctx: &mut Context, timestamp: Timestamp, btc: f64, price: f64, fee: f64, address: &str) -> DataItem {
        let (date, time) = (timestamp.date(), timestamp.time());
        let nano_btc = &format_nano_btc(btc * NANS);
        let usd_fmt = &format_usd(btc*price);
        let price_fmt = &format_usd(price);
        let fee = fee*price;
        let total = format_usd(fee+(btc*price));
        let fee_fmt = format_usd(fee);

        let details: Vec<(&str, &str)> = vec![
            ("Date", &date),
            ("Time", &time),
            ("Amount sent", usd_fmt),
            ("Bitcoin sent", nano_btc),
            ("Bitcoin price", price_fmt),
            ("Sent to address", address),
            ("", ""), // temp spacer
            ("Network fee", &fee_fmt),
            ("Total", &total)
        ];

        DataItem::new(ctx, None, "Transaction details", None, None, Some(details), None)
    }
}