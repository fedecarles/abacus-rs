use crate::utils::deserialize_date;
use chrono::prelude::NaiveDate;
use serde::{Deserialize, Serialize};
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_new() {
        let date = NaiveDate::from_ymd_opt(2023, 10, 13).unwrap();
        let account = "Account1".to_string();
        let payee = Some("Payee1".to_string());
        let quantity = 100.0;
        let amount = 500.0;
        let offset_account = "Account2".to_string();
        let offset_amount = 500.0;
        let note = Some("Note1".to_string());

        let transaction = Transaction::new(
            date,
            account.clone(),
            payee.clone(),
            quantity,
            amount,
            offset_account.clone(),
            offset_amount,
            note.clone(),
        );

        assert_eq!(transaction.date, date);
        assert_eq!(transaction.account, account);
        assert_eq!(transaction.payee, payee);
        assert_eq!(transaction.quantity, quantity);
        assert_eq!(transaction.amount, amount);
        assert_eq!(transaction.offset_account, offset_account);
        assert_eq!(transaction.offset_amount, offset_amount);
        assert_eq!(transaction.note, note);
    }

    #[test]
    fn test_transaction_display() {
        let date = NaiveDate::from_ymd_opt(2023, 10, 13).unwrap();
        let account = "Account1".to_string();
        let payee = Some("Payee1".to_string());
        let quantity = 100.0;
        let amount = 500.0;
        let offset_account = "Account2".to_string();
        let offset_amount = 500.0;
        let note = Some("Note1".to_string());

        let transaction = Transaction::new(
            date,
            account.clone(),
            payee.clone(),
            quantity,
            amount,
            offset_account.clone(),
            offset_amount,
            note.clone(),
        );

        let expected_display = format!(
            "{:<20} * {:<} - {:<}\n{:<10}:  {:>} qty: {:<}\n{:<10}: {:>}\n",
            date,
            payee.clone().unwrap_or_default(),
            note.clone().unwrap_or_default(),
            account,
            amount,
            quantity,
            offset_account,
            offset_amount
        );

        assert_eq!(transaction.to_string(), expected_display);
    }
}
