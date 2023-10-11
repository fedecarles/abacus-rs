use chrono::prelude::*;
use std::fmt;

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub enum AccountType {
    Asset,
    Income,
    Liability,
    Expense,
    Equity,
    Stock,
    MutualFund,
    Holding,
    Cash,
    Unknown,
}

impl fmt::Display for AccountType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            AccountType::Asset => write!(f, "{:<11}", "Asset"),
            AccountType::Income => write!(f, "{:<11}", "Income"),
            AccountType::Liability => write!(f, "{:<11}", "Liability"),
            AccountType::Expense => write!(f, "{:<11}", "Expense"),
            AccountType::Equity => write!(f, "{:<11}", "Equity"),
            AccountType::Stock => write!(f, "{:<11}", "Stock"),
            AccountType::MutualFund => write!(f, "{:<11}", "MutualFund"),
            AccountType::Holding => write!(f, "{:<11}", "Holding"),
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
            account_type: AccountType::Asset,
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
        "Asset" => AccountType::Asset,
        "Income" => AccountType::Income,
        "Liability" => AccountType::Liability,
        "Expense" => AccountType::Expense,
        "Equity" => AccountType::Equity,
        "Stock" => AccountType::Stock,
        "MutualFund" => AccountType::MutualFund,
        "Holding" => AccountType::Holding,
        "Cash" => AccountType::Cash,
        _ => AccountType::Unknown,
    }
}
