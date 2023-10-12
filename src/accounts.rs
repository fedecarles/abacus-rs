use chrono::prelude::*;
use std::fmt;

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub enum AccountType {
    Assets,
    Income,
    Liabilities,
    Expenses,
    Equity,
    Stocks,
    MutualFunds,
    Holdings,
    Cash,
    Unknown,
}

impl fmt::Display for AccountType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            AccountType::Assets => write!(f, "{:<11}", "Assets"),
            AccountType::Income => write!(f, "{:<11}", "Income"),
            AccountType::Liabilities => write!(f, "{:<11}", "Liabilities"),
            AccountType::Expenses => write!(f, "{:<11}", "Expenses"),
            AccountType::Equity => write!(f, "{:<11}", "Equity"),
            AccountType::Stocks => write!(f, "{:<11}", "Stocks"),
            AccountType::MutualFunds => write!(f, "{:<11}", "MutualFunds"),
            AccountType::Holdings => write!(f, "{:<11}", "Holdings"),
            AccountType::Cash => write!(f, "{:<11}", "Cash"),
            AccountType::Unknown => write!(f, "{:<11}", "Unknown"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Account {
    pub name: String,
    pub open: NaiveDate,
    pub currency: String,
    pub account_type: AccountType,
    pub opening_balance: Option<f32>,
}

impl Default for Account {
    fn default() -> Self {
        Self {
            name: String::from("new_account"),
            open: Local::now().date_naive(),
            currency: String::from("USD"),
            account_type: AccountType::Assets,
            opening_balance: None,
        }
    }
}

impl Account {
    pub fn new(
        name: String,
        open: NaiveDate,
        currency: String,
        account_type: AccountType,
        opening_balance: Option<f32>,
    ) -> Self {
        Self {
            name: name.replace('"', ""),
            open: open,
            currency: currency.replace('"', ""),
            account_type: account_type,
            opening_balance: opening_balance,
        }
    }
}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "| {} | {} | {:?} | {} |",
            self.open, self.name, self.account_type, self.currency
        )
    }
}

pub fn account_to_enum(account_type: &str) -> AccountType {
    match account_type {
        "Assets" => AccountType::Assets,
        "Income" => AccountType::Income,
        "Liabilities" => AccountType::Liabilities,
        "Expenses" => AccountType::Expenses,
        "Equity" => AccountType::Equity,
        "Stocks" => AccountType::Stocks,
        "MutualFunds" => AccountType::MutualFunds,
        "Holdings" => AccountType::Holdings,
        "Cash" => AccountType::Cash,
        _ => AccountType::Unknown,
    }
}
