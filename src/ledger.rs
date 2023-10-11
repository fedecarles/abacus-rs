use crate::accounts::*;
use crate::price::Price;
use crate::transaction::Transaction;
use crate::utils::*;
use chrono::prelude::*;
use std::any::{type_name, Any};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::str::FromStr;
use std::string::ParseError;
use toml::Value;

#[derive(Debug)]
pub struct Ledger {
    accounts: Vec<Account>,
    transactions: Vec<Transaction>,
    prices: Vec<Price>,
}

impl Ledger {
    pub fn new(ledger_file: &str) -> Result<Self, Box<dyn Error>> {
        let parsed_toml: Value = toml::from_str(&ledger_file).expect("Failed to parse TOML");

        let account_list = parsed_toml.get("account").and_then(|v| v.as_array());
        let transactions_list = parsed_toml.get("transaction").and_then(|v| v.as_array());
        let prices_list = parsed_toml.get("price").and_then(|v| v.as_array());

        Ok(Self {
            accounts: Self::_get_accounts(account_list)?,
            transactions: Self::_get_transactions(transactions_list)?,
            prices: Self::_get_prices(prices_list)?,
        })
    }

    fn _get_accounts(account_list: Option<&Vec<Value>>) -> Result<Vec<Account>, String> {
        let all_accounts: Vec<Account> = match account_list {
            Some(list) => {
                let mut accounts = Vec::new();

                for account in list.iter() {
                    let name = parse_value_to_string(account, "name");
                    let open = parse_value_to_naivedate(account, "open");
                    let currency = parse_value_to_string(account, "currency");
                    let account_type = parse_value_to_slice(account, "type");
                    let opening_balance = match account.get("opening_balance") {
                        Some(f) => f.to_string().parse::<f32>().ok(),
                        None => None,
                    };

                    let account = Account::new(
                        name,
                        open,
                        currency,
                        account_to_enum(account_type),
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

    fn _get_transactions(
        transactions_list: Option<&Vec<Value>>,
    ) -> Result<Vec<Transaction>, String> {
        let all_transactions: Result<Vec<Transaction>, String> = match transactions_list {
            Some(list) => {
                let mut transactions = Vec::new();

                for transaction in list.iter() {
                    let account = parse_value_to_string(transaction, "account");
                    let date = parse_value_to_naivedate(transaction, "date");
                    let payee = match transaction.get("payee") {
                        Some(p) => Some(p.to_string().replace('"', "")),
                        None => None,
                    };
                    let quantity = match transaction.get("quantity") {
                        Some(q) => q.as_float().map(|f| f as f32),
                        None => Some(1.0),
                    }
                    .unwrap_or(1.0);
                    let amount = parse_value_to_f32(transaction, "amount");
                    let offset_account = parse_value_to_string(transaction, "offset_account");
                    let offset_amount = match transaction.get("offset_amount") {
                        Some(q) => q.as_float().unwrap_or_default() as f32,
                        None => amount * -1.0,
                    };
                    let note = match transaction.get("note") {
                        Some(n) => Some(n.to_string().replace('"', "")),
                        None => None,
                    };

                    let transaction = Transaction::new(
                        date,
                        account,
                        payee,
                        quantity,
                        amount,
                        offset_account,
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

    fn _get_prices(price_list: Option<&Vec<Value>>) -> Result<Vec<Price>, String> {
        let all_prices: Result<Vec<Price>, String> = match price_list {
            Some(list) => {
                let mut prices = Vec::new();

                for price in list.iter() {
                    let date = parse_value_to_naivedate(price, "date");
                    let commodity = parse_value_to_string(price, "commodity");
                    let amount = parse_value_to_f32(price, "price");
                    let currency = parse_value_to_string(price, "currency");
                    let price = Price::new(date, commodity, amount, currency);
                    prices.push(price);
                }
                Ok(prices)
            }
            None => Ok(Vec::new()),
        };
        all_prices
    }

    pub fn print_journal(
        &mut self,
        year: Option<String>,
        account_type: Option<String>,
        name: Option<String>,
        payee: Option<String>,
    ) {
        &self.transactions.sort_by(|a, b| a.date.cmp(&b.date));

        let filtered_transactions: Vec<&Transaction> = match year {
            Some(y) => self._query_by_transaction_date(&y),
            None => self.transactions.iter().collect(),
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

        let name_list: Vec<usize> = self.accounts.iter().map(|a| a.clone().name.len()).collect();
        let name_max: &usize = name_list.iter().max().unwrap();

        for a in filtered_accounts {
            for t in &filtered_transactions {
                if t.account.eq(&a.name) {
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
    }

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

    pub fn print_trial_balances(
        &mut self,
        period: Option<String>,
        account_type: Option<Vec<String>>,
        price: Option<String>,
    ) {
        &self
            .accounts
            .sort_by(|a, b| a.account_type.cmp(&b.account_type));

        let filtered_transactions: Vec<&Transaction> = match period {
            Some(y) => self._query_by_transaction_date(&y),
            None => self.transactions.iter().collect(),
        };

        let filtered_accounts: Vec<&Account> = match account_type {
            Some(a) => a
                .iter()
                .flat_map(|atype| self._query_by_account_type(atype))
                .collect(),
            None => self.accounts.iter().collect(),
        };
        let name_list: Vec<usize> = filtered_accounts
            .iter()
            .map(|a| a.clone().name.len())
            .collect();
        let name_max: &usize = name_list.iter().max().unwrap();

        let prices: &Vec<Price> = &self.prices;

        let bal = &self._get_balances(price.to_owned());

        let mut types: Vec<&AccountType> =
            filtered_accounts.iter().map(|t| &t.account_type).collect();
        types.dedup();
        let is_zero = 0.0 as f32;

        for t in types {
            println!("{}", t);
            for a in &filtered_accounts {
                if t == &a.account_type {
                    for (account, amount) in bal.iter() {
                        if (account.eq(&a.name)) & (amount.ne(&is_zero)) {
                            let curr = match &price {
                                Some(p) => p,
                                None => &a.currency,
                            };
                            let output = format!(
                                "    {:<name_width$} {:11.2} {}",
                                a.name.replace('"', ""),
                                amount,
                                curr.replace('"', ""),
                                name_width = name_max,
                            );
                            println!("{}", output);
                        }
                    }
                }
            }
        }
    }

    fn _get_balances(&self, price: Option<String>) -> HashMap<String, f32> {
        let mut balances: HashMap<String, f32> = HashMap::new();
        for a in &self.accounts {
            let opening_balances = balances.entry(a.name.clone()).or_insert(0.0);
            *opening_balances += a.opening_balance.unwrap_or_default();
        }
        for t in &self.transactions {
            let amounts = balances.entry(t.account.clone()).or_insert(0.0);
            *amounts += t.amount * t.quantity;
            let offsets = balances.entry(t.offset_account.clone()).or_insert(0.0);
            *offsets += t.offset_amount;
        }

        if let Some(p) = price {
            let mut relevant_prices: HashMap<String, f32> = HashMap::new();
            let mut x = self.prices.clone();
            let mut selected_currency: Vec<&Price> =
                x.iter().filter(|c| c.currency.eq(&p)).collect();
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

    pub fn _query_by_account_name(&self, account_name: &str) -> Vec<&Account> {
        return self
            .accounts
            .iter()
            .filter(|a| a.name.as_str().eq(account_name))
            .collect();
    }

    pub fn _query_by_account_type(&self, account_type: &str) -> Vec<&Account> {
        return self
            .accounts
            .iter()
            .filter(|a| a.account_type.eq(&account_to_enum(account_type)))
            .collect();
    }

    pub fn _query_by_account_currency(&self, account_currency: &str) -> Vec<&Account> {
        return self
            .accounts
            .iter()
            .filter(|a| a.currency.as_str().eq(account_currency))
            .collect();
    }

    pub fn _query_by_account_date(&self, account_date: &str) -> Vec<&Account> {
        let query_year =
            NaiveDate::from_str(&format!("{}-01-01", account_date)).unwrap_or_default();
        return self
            .accounts
            .iter()
            .filter(|a| a.open.year().eq(&query_year.year()))
            .collect();
    }

    pub fn _query_by_transaction_payee(&self, payee: &str) -> Vec<&Transaction> {
        return self
            .transactions
            .iter()
            .filter(|t| t.payee.eq(&Some(payee.to_string())))
            .collect();
    }

    pub fn _query_by_transaction_date(&self, year: &str) -> Vec<&Transaction> {
        let query_year = NaiveDate::from_str(&format!("{}-01-01", year)).unwrap_or_default();
        let filtered_transactions: Vec<&Transaction> = self
            .transactions
            .iter()
            .filter(|t| t.date.year().eq(&query_year.year()))
            .collect();
        return filtered_transactions;
    }
}
