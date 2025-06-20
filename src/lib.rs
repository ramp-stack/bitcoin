use bdk_wallet::bitcoin::{Amount, Txid};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Local};

pub mod wallet;
pub mod components;
pub mod events;
pub mod pages;
pub mod service;

pub const NANS: f64 = 1_000_000_000.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BDKTransaction {
    pub datetime: Option<DateTime<Local>>,
    pub txid: Txid,
    pub is_received: bool,
    pub amount: Amount,
    pub price: f64,
    pub fee: Option<Amount>,
    pub address: Option<String>
}

pub fn format_usd(t: f64) -> String {
    let mut dollars = t.trunc() as u64;
    let mut cents = (t.fract() * 100.0).round() as u64;

    if cents == 100 {
        dollars += 1;
        cents = 0;
    }

    let dollar_str = dollars.to_string();
    let mut chars = dollar_str.chars().rev().collect::<Vec<_>>();
    for i in (3..chars.len()).step_by(3) {
        chars.insert(i, ',');
    }
    let formatted_dollars = chars.into_iter().rev().collect::<String>();

    format!("${}.{:02}", formatted_dollars, cents)
}


pub fn format_nano_btc(nb: f64) -> String {
    let rounded = nb.round() as u64;
    let formatted = rounded.to_string().chars().rev().enumerate()
        .flat_map(|(i, c)| {if i != 0 && i % 3 == 0 {vec![',', c]} else {vec![c]}})
        .collect::<Vec<_>>().into_iter().rev().collect::<String>();

    format!("{} nb", formatted)
}


pub fn format_address(a: String) -> String {
    format!("{}...{}", &a[..7], &a[a.len().saturating_sub(3)..])
}

