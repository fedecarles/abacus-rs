use crate::utils::deserialize_date;
use chrono::prelude::*;
use serde::{Deserialize, Deserializer};
use serde_derive::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Deserialize, Serialize)]
pub struct Transaction {
    #[serde(deserialize_with = "deserialize_date", skip_serializing)]
    pub date: NaiveDate,
    pub account: String,
    pub note: Option<String>,
    pub payee: Option<String>,
    pub quantity: f32,
    pub amount: f32,
    pub offset_account: String,
    pub offset_amount: f32,
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let header = format!(
            "{:<20} * {:<} - {:<}",
            self.date,
            self.payee.clone().unwrap_or_default(),
            self.note.clone().unwrap_or_default()
        );
        let posting = format!(
            "{:<10}:  {:>} qty: {:<}",
            self.account, self.amount, self.quantity
        );
        let offset = format!("{:<10}: {:>}", self.offset_account, self.offset_amount);
        write!(f, "{}\n{}\n{}\n", header, posting, offset)
    }
}

impl Transaction {
    pub fn new(
        date: NaiveDate,
        account: String,
        payee: Option<String>,
        quantity: f32,
        amount: f32,
        offset_account: String,
        offset_amount: f32,
        note: Option<String>,
    ) -> Self {
        Self {
            date: date,
            account: account.replace('"', ""),
            payee: payee,
            quantity: quantity,
            amount: amount,
            offset_account: offset_account.replace('"', ""),
            offset_amount: offset_amount,
            note: note,
        }
    }
}
