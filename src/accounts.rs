use chrono::prelude::*;
use std::{fmt, str::FromStr};

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

impl FromStr for AccountType {
    type Err = ();
    fn from_str(input: &str) -> Result<AccountType, Self::Err> {
        match input {
            "Assets" => Ok(AccountType::Assets),
            "Income" => Ok(AccountType::Income),
            "Liabilities" => Ok(AccountType::Liabilities),
            "Expenses" => Ok(AccountType::Expenses),
            "Equity" => Ok(AccountType::Equity),
            "Stocks" => Ok(AccountType::Stocks),
            "MutualFunds" => Ok(AccountType::MutualFunds),
            "Holdings" => Ok(AccountType::Holdings),
            "Cash" => Ok(AccountType::Cash),
            _ => Ok(AccountType::Unknown),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_default() {
        let account = Account::default();
        assert_eq!(account.name, "new_account");
        assert_eq!(account.currency, "USD");
        assert_eq!(account.account_type, AccountType::Assets);
        assert_eq!(account.open, Local::now().date_naive());
        assert_eq!(account.opening_balance, None);
    }

    #[test]
    fn test_account_new() {
        let name = "Test Account".to_string();
        let open = NaiveDate::from_ymd_opt(2023, 10, 13).unwrap();
        let currency = "EUR".to_string();
        let account_type = AccountType::Income;
        let opening_balance = Some(1000.0);

        let account = Account::new(
            name.clone(),
            open,
            currency.clone(),
            account_type.clone(),
            opening_balance,
        );

        assert_eq!(account.name, name);
        assert_eq!(account.currency, currency);
        assert_eq!(account.account_type, account_type);
        assert_eq!(account.open, open);
        assert_eq!(account.opening_balance, opening_balance);
    }

    #[test]
    fn test_account_to_enum() {
        assert_eq!(account_to_enum("Assets"), AccountType::Assets);
        assert_eq!(account_to_enum("Income"), AccountType::Income);
        assert_eq!(account_to_enum("Liabilities"), AccountType::Liabilities);
        assert_eq!(account_to_enum("Expenses"), AccountType::Expenses);
        assert_eq!(account_to_enum("Equity"), AccountType::Equity);
        assert_eq!(account_to_enum("Stocks"), AccountType::Stocks);
        assert_eq!(account_to_enum("MutualFunds"), AccountType::MutualFunds);
        assert_eq!(account_to_enum("Holdings"), AccountType::Holdings);
        assert_eq!(account_to_enum("Cash"), AccountType::Cash);
        assert_eq!(account_to_enum("Unknown"), AccountType::Unknown);
    }
}
