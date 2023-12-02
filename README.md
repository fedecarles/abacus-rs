# Abacus-rs
### A cli plain-text accounting program

This tool was writen for my own purposes, though anyone is welcome to use it or contribute.
But if you are serious about plain-text accounting, please consider using [ledger](https://ledger-cli.org/),
[hledger](https://hledger.org/) or [beancount](https://github.com/beancount/). Or visit [plaintextaccounting.org](https://plaintextaccounting.org/) to learn more about it.

For another rust based approach, you can also check [rust_ledger](https://github.com/ebcrowder/rust_ledger/tree/main), where I took some inspiration from.

## Features
- Double-entry **like** account keeping.
- Uses [toml](https://toml.io/en/) text format.
- Import of transactions from csv.
- Balance and Journal reports.
- Group balances by Month, Quarter or Year.

## Usage

```bash
Usage: abacus-rs --ledger <LEDGER> [COMMAND]

Commands:
  accounts  List accounts
  balances  Print account balance sheet report
  journal   Print transactions journal report
  import    Import transactions from csv
  help      Print this message or the help of the given subcommand(s)

Options:
  -l, --ledger <LEDGER>  Path to ledger file or directory
  -h, --help             Print help
  -V, --version          Print version
```

An abacus ledger is toml file, or mutliple toml files, that include accounts,
transaction and optionaly, prices.

Running any command requires declaring the path to the ledger.

### Accounts

An account is declared with a specific type; Assets, Liabilities,
Expenses, Income, Equity, Stock, MutualFund, Holding or Cash.

The account currency can be declared with any terminology.

```toml
[[account]]
open = 2023-09-30
name = "Savings Account"
type = "Assets"
currency = "USD"

[[account]]
open = 2023-09-30
name = "Dining"
type = "Expenses"
currency = "USD"
```

An optional opening balance can be included.

```toml
[[account]]
open = 2023-09-30
name = "Savings Account"
type = "Assets"
currency = "USD"
opening_balance = 1000.00
```

### Transactions

Transactions require a date (in YYYY-MM-DD format),
an amount (float or integer), an account and an offset_account.
The Offset Amount and Quantity can be explicity declared,
otherwise it will be set as the inverse of the amount 
and to one (1) respectively. Payee and note are optional fields.

```toml
[[transaction]]
date = 2023-10-03
amount = 100.00
account = "Dining"
offset_account = "Savings Account"
offset_amount = -100.00 # optional
quantity = 1            # optional
payee = "RESTAURAN X"   # optional
note = "Meal was good"  # optional
```

### Prices

Declaring Commodity prices is entirely optional but very useful to price
stocks or currencies.


```toml
[[price]]
date = 2023-10-02
commodity = "ARS" 
price = 0.00125
currency = "USD"

[[price]]
date = 2023-09-30
commodity = "VOO"
price = 390.50
currency = "USD"
```

### Print Balances

```bash
Usage: abacus-rs --ledger <LEDGER> balances [OPTIONS]

Options:
  -c, --class [<CLASS>...]  Filter accounts by account type
  -y, --year <YEAR>         Filter transactions by year
  -p, --price <PRICE>       Price balances at specific currency
  -h, --help                Print help
```

Account balances are printed for all accounts by default. 

```bash
Assets
    Savings Account       3245.00 USD
    Crypto Wallet            0.56 BTC
Income
    Salary               -2300.00 USD
Liabilities
    Credit Card           -200.00 USD
Expenses
    Clothes                200.00 USD
    Dining                  55.00 USD
```

Specific account classes can be passed with the -c option to print a more typical
balance sheet view. The amount can also be priced at a specific currency, provided
there is a **price** entry for it in the ledger.

```bash
> abacus-rs -l example/ balances -c Assets Liabilities -p ARS

Assets
    Savings Account    3017850.00 ARS
    Crypto Wallet            0.56 ARS
Liabilities
    Credit Card        -186000.00 ARS
```

### Print Journal

```bash
Print transactions journal report

Usage: abacus-rs --ledger <LEDGER> journal [OPTIONS]

Options:
  -y, --year <YEAR>        Filter transactions by year
  -c, --class <CLASS>      Filter accounts by account type
  -a, --account <ACCOUNT>  Filter accounts by account name
  -p, --payee <PAYEE>      Filter transactions by payee
  -h, --help               Print help
```
A journal is a list of the existing transactions in the ledger. 

Transactions can be filtered by year, account class, name or payee.

Example for listing only Dining expenses transacions. 

```bash
> abacus-rs -l example/ journal -a Dining

2023-10-10 | Dining             |       20.00 | RESTAURANT X
2023-10-10 | Savings Account    |      -20.00 |
2023-10-11 | Dining             |       35.00 | RESTAURANT Y
2023-10-11 | Savings Account    |      -35.00 |
```

### Import transactions

```bash
Import transactions from csv

Usage: abacus-rs --ledger <LEDGER> import [OPTIONS] --csv <CSV>

Options:
  -c, --csv <CSV>        CSV file with transactions to import
  -f, --format <FORMAT>  Date format
  -h, --help             Print help
```

To import transactions from a csv file some pre-formatting needs to be done
to ensure the columns names are mapped with the same names as in the toml files.

Some banks provide statements in credit/debit format and other with sign or 
parenthesis for negative values, so some number formatting may also be required.

Running the import requires specifying the file where the toml transaction
are to be imported and the csv file with the details.

Example of running the import script.

```bash
> abacus-rs -l example/transactions.toml import -c ~/Downloads/bbva/visa/sep23.csv
Import start
Imported: CsvRow { date: "2023-09-28", account: "Taxes", ...
Imported: CsvRow { date: "2023-09-28", account: "Taxes", ...
Imported: CsvRow { date: "2023-09-28", account: "Taxes", ...
Imported: CsvRow { date: "2023-09-28", account: "Taxes", ...
Imported: CsvRow { date: "2023-09-23", account: "Dining",...
Imported: CsvRow { date: "2023-09-23", account: "Dining",...
Imported: CsvRow { date: "2023-09-23", account: "Medical ...
Import complete
```

