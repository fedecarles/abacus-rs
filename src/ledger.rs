//! This module defines the main [Ledger] struct and operations.

use crate::accounts::*;
use crate::price::Price;
use crate::transaction::Transaction;
use crate::utils::*;
use chrono::prelude::*;
use itertools::Itertools;
use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;
use toml::Value;

#[derive(Debug)]
pub struct Ledger {
    accounts: Vec<Account>,
    transactions: Vec<Transaction>,
    prices: Vec<Price>,
}

impl Ledger {
    pub fn new(ledger_file: &str) -> Result<Self, Box<dyn Error>> {
        let parsed_toml: Value = toml::from_str(&ledger_file)?;

        let account_list = parsed_toml.get("account").and_then(|v| v.as_array());
        let transactions_list = parsed_toml.get("transaction").and_then(|v| v.as_array());
        let prices_list = parsed_toml.get("price").and_then(|v| v.as_array());

        Ok(Self {
            accounts: Self::_get_accounts(account_list)?,
            transactions: Self::_get_transactions(transactions_list)?,
            prices: Self::_get_prices(prices_list)?,
        })
    }

    /// Parses the accounts from the ledger file.
    fn _get_accounts(account_list: Option<&Vec<Value>>) -> Result<Vec<Account>, String> {
        let all_accounts: Vec<Account> = match account_list {
            Some(list) => {
                let mut accounts = Vec::new();

                for account in list.iter() {
                    let name = parse_value(account, "name");
                    let open = parse_value(account, "open");
                    let currency = parse_value(account, "currency");
                    let account_type = parse_value(account, "type");
                    let opening_balance = match account.get("opening_balance") {
                        Some(f) => f.to_string().parse::<f32>().ok(),
                        None => None,
                    };
                    let account = Account::new(
                        name.unwrap_or_default(),
                        open.unwrap_or_default(),
                        currency.unwrap_or_default(),
                        account_type.unwrap_or(AccountType::Assets),
                        opening_balance,
                    );

                    accounts.push(account);
                }
                accounts
            }
            None => Vec::new(),
        };
        Ok(all_accounts)
    }

    /// Parses the transactions from the ledger file.
    fn _get_transactions(
        transactions_list: Option<&Vec<Value>>,
    ) -> Result<Vec<Transaction>, String> {
        let all_transactions: Result<Vec<Transaction>, String> = match transactions_list {
            Some(list) => {
                let mut transactions = Vec::new();

                for transaction in list.iter() {
                    let account = parse_value(transaction, "account");
                    let date = parse_value_to_naivedate(transaction, "date");
                    let payee = parse_value(transaction, "payee");
                    let quantity = match transaction.get("quantity") {
                        Some(q) => q.as_float().map(|f| f as f32),
                        None => Some(1.0),
                    };
                    let amount = parse_value_to_f32::<f32>(transaction, "amount");
                    let offset_account = parse_value(transaction, "offset_account");
                    let offset_amount = match transaction.get("offset_amount") {
                        Some(q) => q.as_float().unwrap_or_default() as f32,
                        None => amount.unwrap_or_default() * -1.0,
                    };
                    let note = parse_value(transaction, "note");

                    let transaction = Transaction::new(
                        date.unwrap_or_default(),
                        account.unwrap_or_default(),
                        payee,
                        quantity.unwrap_or(1.0),
                        amount.unwrap_or_default(),
                        offset_account.unwrap_or_default(),
                        offset_amount,
                        note,
                    );
                    transactions.push(transaction);
                }
                Ok(transactions)
            }
            None => Ok(Vec::new()),
        };
        all_transactions
    }

    /// Parses the commodity prices from the ledger file.
    fn _get_prices(price_list: Option<&Vec<Value>>) -> Result<Vec<Price>, String> {
        let all_prices: Result<Vec<Price>, String> = match price_list {
            Some(list) => {
                let mut prices = Vec::new();

                for price in list.iter() {
                    let date = parse_value_to_naivedate(price, "date");
                    let commodity = parse_value(price, "commodity");
                    let amount = parse_value_to_f32::<f32>(price, "price");
                    let currency = parse_value(price, "currency");
                    let price = Price::new(
                        date.unwrap_or_default(),
                        commodity.unwrap_or_default(),
                        amount.unwrap_or_default(),
                        currency.unwrap_or_default(),
                    );
                    prices.push(price);
                }
                Ok(prices)
            }
            None => Ok(Vec::new()),
        };
        all_prices
    }

    /// Validates each transaction in the ledger:
    /// 1. For each transaction, check if the account is declared.
    /// 2. For each transaction, check if the amounts are balanced.
    /// 3. Transactions between accounts with different currencies are not validated for balance.
    pub fn validate_transactions(&self) {
        for t in self.transactions.iter() {
            // check if the account exists
            let account_exists = self.accounts.iter().any(|a| a.name == t.account);
            if !account_exists {
                panic!("Account {} does not exist", t.account)
            }

            // check if transactions balances
            let sum_postings = t.amount + t.offset_amount;
            if sum_postings != 0.0 {
                // only check check balances if the accounts have the same currency
                let account_currency = &self.accounts.iter().find(|a| a.name == t.account);
                let offset_currency = &self.accounts.iter().find(|a| a.name == t.offset_account);

                if let Some(ac) = account_currency {
                    if let Some(oc) = offset_currency {
                        if ac.currency.eq(&oc.currency) {
                            panic!("Transaction does not balance:\n {}", t)
                        }
                    }
                }
            }
        }
    }

    /// Print a journal of transactions.
    pub fn print_journal(
        &mut self,
        from: Option<String>,
        to: Option<String>,
        account_type: Option<String>,
        name: Option<String>,
        payee: Option<String>,
    ) {
        self.transactions.sort_by(|a, b| a.date.cmp(&b.date));
        self.validate_transactions();

        let filtered_transactions: Vec<&Transaction> = match (from, to) {
            (Some(f), Some(t)) => self._query_by_transaction_date(Some(&f), Some(&t)),
            (Some(f), None) => self._query_by_transaction_date(Some(&f), None),
            (None, Some(t)) => self._query_by_transaction_date(None, Some(&t)),
            (None, None) => self.transactions.iter().collect(),
        };

        let filtered_transactions: Vec<&Transaction> = match payee {
            Some(p) => self._query_by_transaction_payee(&p),
            None => filtered_transactions,
        };

        let filtered_accounts: Vec<&Account> = match account_type {
            Some(t) => self._query_by_account_type(&t),
            None => self.accounts.iter().collect(),
        };
        let filtered_accounts: Vec<&Account> = match name {
            Some(n) => self._query_by_account_name(&n),
            None => filtered_accounts,
        };

        let name_list: Vec<usize> = self.accounts.iter().map(|a| a.name.len()).collect();
        let name_max: &usize = name_list.iter().max().unwrap();

        for t in &filtered_transactions {
            let get_account = filtered_accounts
                .iter()
                .find(|a| (a.name == t.account) | (a.name == t.offset_account));
            if get_account.is_some() {
                let posting = format!(
                    "{} | {:<name_width$} | {:11.2} | {}",
                    t.date,
                    t.account,
                    t.amount,
                    t.payee.clone().unwrap_or_default(),
                    name_width = name_max + 1
                );
                let offset = format!(
                    "{} | {:<name_width$} | {:11.2} |",
                    t.date,
                    t.offset_account,
                    t.offset_amount,
                    name_width = name_max + 1
                );
                println!("{}\n{}", posting, offset);
            }
        }
    }

    /// Print a list of all declared accounts.
    pub fn print_accounts(self) {
        let name_list: Vec<usize> = self.accounts.iter().map(|a| a.name.len()).collect();
        let name_max: &usize = name_list.iter().max().unwrap();
        for a in self.accounts {
            let output = format!(
                "| {} | {} | {:<name_width$} | {}",
                a.open,
                a.account_type,
                a.name.replace('"', ""),
                a.currency.replace('"', ""),
                name_width = name_max,
            );
            println!("{}", output);
        }
    }

    /// Print a list of account balances.
    pub fn print_balances(
        &mut self,
        from: Option<String>,
        to: Option<String>,
        account_type: Option<Vec<String>>,
        price: Option<String>,
        group: Option<String>,
    ) {
        self.validate_transactions();

        let mut filtered_transactions: Vec<&Transaction> = match (from, to) {
            (Some(f), Some(t)) => self._query_by_transaction_date(Some(&f), Some(&t)),
            (Some(f), None) => self._query_by_transaction_date(Some(&f), None),
            (None, Some(t)) => self._query_by_transaction_date(None, Some(&t)),
            (None, None) => self.transactions.iter().collect(),
        };

        // Get all potential account names
        let filtered_accounts: Vec<&Account> = match account_type {
            Some(a) => a
                .iter()
                .flat_map(|atype| self._query_by_account_type(atype))
                .collect(),
            None => self.accounts.iter().collect(),
        };

        filtered_transactions.sort_by(|a, b| a.date.cmp(&b.date));

        let balances_by_period =
            self._group_transactions_by_period(filtered_transactions, price.to_owned(), group);

        let sorted_periods: Vec<_> = balances_by_period
            .keys()
            .sorted_by(|a, b| b.cmp(&a))
            .collect();

        // Get account names with actual balances
        let mut account_names: Vec<String> = balances_by_period
            .values()
            .flat_map(|balances| balances.keys().cloned())
            .collect();

        account_names.sort();
        account_names.dedup();

        // find the max lenght of the account names
        let name_max: Option<usize> = filtered_accounts.iter().map(|a| a.name.len()).max();

        let mut atypes: Vec<_> = filtered_accounts.iter().map(|t| &t.account_type).collect();
        atypes.dedup();

        // Begin printing balances
        // Print header
        let header = format!(
            "{:<name_width$}",
            "Accounts",
            name_width = name_max.unwrap_or(15)
        );
        print!("\t{:>} ", header);
        for h in &sorted_periods {
            print!("\t{:>15}-{}", h.0, h.1);
        }
        println!("");

        // Print data rows
        for t in atypes {
            println!("{}", t);

            for a in filtered_accounts
                .iter()
                .filter(|a| account_names.contains(&&a.name) && t.eq(&a.account_type))
            {
                let name = format!(
                    "{:<name_width$}",
                    a.name,
                    name_width = name_max.unwrap_or(15)
                );
                print!("\t{:<15}", name);
                for p in &sorted_periods {
                    if let Some(period_data) = balances_by_period.get(p) {
                        if let Some(value) = period_data.get(&a.name) {
                            print!(
                                "\t{:>15.2} {}",
                                value,
                                &price.as_ref().unwrap_or(&a.currency)
                            );
                        } else {
                            print!("\t{:>15.2} {}", 0.0, &price.as_ref().unwrap_or(&a.currency));
                        }
                    } else {
                        print!("\t{:>15.2} {}", 0.0, a.currency);
                    }
                }
                println!("");
            }
        }
    }

    /// Calculates the balance amounts.
    fn _get_balances(
        &self,
        transactions: Vec<&Transaction>,
        price: Option<String>,
    ) -> HashMap<String, f32> {
        let mut balances: HashMap<String, f32> = HashMap::new();
        for a in &self.accounts {
            let opening_balances = balances.entry(a.name.clone()).or_insert(0.0);
            *opening_balances += a.opening_balance.unwrap_or_default();
        }
        for t in &transactions {
            let amounts = balances.entry(t.account.clone()).or_insert(0.0);
            *amounts += t.amount * t.quantity;
            let offsets = balances.entry(t.offset_account.clone()).or_insert(0.0);
            *offsets += t.offset_amount;
        }

        if let Some(p) = price {
            let mut relevant_prices: HashMap<String, f32> = HashMap::new();
            let mut selected_currency: Vec<&Price> =
                self.prices.iter().filter(|c| c.currency.eq(&p)).collect();
            selected_currency.sort_by(|a, b| b.date.cmp(&a.date));

            for p in selected_currency {
                let commodity = p.commodity.clone();
                let last_price = p.price;
                relevant_prices.entry(commodity).or_insert(last_price);
            }

            for a in &self.accounts {
                for (commodity, price) in relevant_prices.iter() {
                    if commodity.eq(&a.currency) {
                        let pricings = balances.entry(a.name.clone()).or_insert(0.0);
                        *pricings *= price;
                    }
                }
            }
        }
        balances
    }

    /// Aggregates balances by date grouping.
    fn _group_transactions_by_period(
        &self,
        transactions: Vec<&Transaction>,
        price: Option<String>,
        group: Option<String>,
    ) -> HashMap<(u32, u32), HashMap<String, f32>> {
        // Create a HashMap to store data for each period
        let mut transactions_by_period: HashMap<(u32, u32), Vec<&Transaction>> = HashMap::new();

        // Iterate through the transactions and categorize data by period
        for entry in transactions {
            let month = entry.date.month();
            let quarter = quarter(entry.date.month());
            let year = entry.date.year() as u32;

            let period: (u32, u32) = match group {
                Some(ref g) => match g.as_str() {
                    "M" => (year, month),
                    "Q" => (year, quarter),
                    "Y" => (year, year),
                    _ => (0, 0),
                },
                None => (0, 0),
            };

            // Add the entry to the corresponding month in the HashMap
            transactions_by_period
                .entry(period)
                .or_insert_with(Vec::new)
                .push(entry);
        }

        let mut balances_by_period: HashMap<(u32, u32), HashMap<String, f32>> = HashMap::new();

        // Get balances for each period
        for (period, transactions) in transactions_by_period {
            let mut bal = self._get_balances(transactions, price.to_owned());
            bal.retain(|_, &mut value| value != 0.0);
            balances_by_period.entry(period).or_insert(bal);
        }
        balances_by_period
    }

    /// Filter accounts by name.
    pub fn _query_by_account_name(&self, account_name: &str) -> Vec<&Account> {
        return self
            .accounts
            .iter()
            .filter(|a| a.name.as_str().eq(account_name))
            .collect();
    }

    /// Filter accounts by class type.
    pub fn _query_by_account_type(&self, account_type: &str) -> Vec<&Account> {
        return self
            .accounts
            .iter()
            .filter(|a| {
                a.account_type
                    .eq(&AccountType::from_str(account_type).unwrap_or(AccountType::Assets))
            })
            .collect();
    }

    /// Filter accounts by currency.
    pub fn _query_by_account_currency(&self, account_currency: &str) -> Vec<&Account> {
        return self
            .accounts
            .iter()
            .filter(|a| a.currency.as_str().eq(account_currency))
            .collect();
    }

    /// Filter transactions by payee.
    pub fn _query_by_transaction_payee(&self, payee: &str) -> Vec<&Transaction> {
        return self
            .transactions
            .iter()
            .filter(|t| t.payee.eq(&Some(payee.to_string())))
            .collect();
    }

    /// Filter transactions by from and to dates.
    pub fn _query_by_transaction_date(
        &self,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Vec<&Transaction> {
        let from = NaiveDate::from_str(from.unwrap_or("1970-01-01")).unwrap_or_default();
        let to = NaiveDate::from_str(to.unwrap_or("2999-01-01")).unwrap_or_default();
        let filtered_transactions: Vec<&Transaction> = self
            .transactions
            .iter()
            .filter(|t| t.date.ge(&from) && t.date.le(&to))
            .collect();
        filtered_transactions
    }
}
