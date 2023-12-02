//! # Abacus-rs
//! Abacus-rs is a simplified command line accounting tool inspired by more
//! robust programs like [ledger](https://ledger-cli.org/),
//! [hledger](https://hledger.org/) and [beancount](https://github.com/beancount/).
//!
//! # Features
//!
//! - Double-entry *like* account keeping.
//! - Uses [toml](https://toml.io/en/) format for the ledger.
//! - Import of transactions from csv.
//! - Balance and Journal reports.
//! - Grouping by month, quarter or year.
//! - Commodity pricing.
//!
//! # Usage
//!
//! ```bash
//! Usage: abacus-rs --ledger <LEDGER> [COMMAND]
//!
//! Commands:
//!   accounts  List accounts
//!   balances  Print account balance sheet report
//!   journal   Print transactions journal report
//!   import    Import transactions from csv
//!   help      Print this message or the help of the given subcommand(s)
//!
//! Options:
//!   -l, --ledger <LEDGER>  Path to ledger file or directory
//!   -h, --help             Print help
//!   -V, --version          Print version
//! ```
//!
//! An abacus ledger is a toml file, or mutliple toml files, that include accounts,
//! transaction and optionaly, commodity prices.
//!
//! Running any command requires declaring the path to the ledger.
//!
//! ### Print Balances
//!
//! ```bash
//! Usage: abacus-rs --ledger <LEDGER> balances [OPTIONS]
//! Options:
//!   -c, --class [<CLASS>...]  Filter accounts by account type
//!   -f, --from <FROM>         Filter transactions by start date
//!   -t, --to <TO>             Filter transactions by end date
//!   -p, --price <PRICE>       Price balances at specific currency
//!   -g, --group <GROUP>       Group balances by period (M, Q or Y)
//!   -h, --help                Print help
//!
//! Account balances are printed for all accounts by default.
//!
//! ```bash
//! Assets
//!     Savings Account       3245.00 USD
//!     Crypto Wallet            0.56 BTC
//! Income
//!     Salary               -2300.00 USD
//! Liabilities
//!     Credit Card           -200.00 USD
//! Expenses
//!     Clothes                200.00 USD
//!     Dining                  55.00 USD
//! ```
//!
//! Specific account classes can be passed with the -c option to print a more typical
//! balance sheet view. The amount can also be priced at a specific currency, provided
//! there is a **price** entry for it in the ledger.
//!
//! ```bash
//! > abacus-rs -l example/ balances -c Assets Liabilities -p ARS
//!
//! Assets
//!     Savings Account    3017850.00 ARS
//!     Crypto Wallet            0.56 ARS
//! Liabilities
//!     Credit Card        -186000.00 ARS
//! ```
//!
//! ### Print Journal
//!
//! ```bash
//! Print transactions journal report
//!
//! Usage: abacus-rs --ledger <LEDGER> journal [OPTIONS]
//!
//! Options:
//!   -f, --from <FROM>        Filter transactions by start date
//!   -t, --to <TO>            Filter transactions by end date
//!   -c, --class <CLASS>      Filter accounts by account type
//!   -a, --account <ACCOUNT>  Filter accounts by account name
//!   -p, --payee <PAYEE>      Filter transactions by payee
//!   -h, --help               Print help
//!  ```
//! A journal is a list of the existing transactions in the ledger.
//!
//! Transactions can be filtered by date, account class, name or payee.
//!
//! Example for listing only Dining expenses transacions.
//!
//! ```bash
//! > abacus-rs -l example/ journal -a Dining
//!
//! 2023-10-10 | Dining             |       20.00 | RESTAURANT X
//! 2023-10-10 | Savings Account    |      -20.00 |
//! 2023-10-11 | Dining             |       35.00 | RESTAURANT Y
//! 2023-10-11 | Savings Account    |      -35.00 |
//! ```
//!
//! ### Import transactions
//!
//! ```bash
//! Import transactions from csv
//!
//! Usage: abacus-rs --ledger <LEDGER> import [OPTIONS] --csv <CSV>
//!
//! Options:
//!   -c, --csv <CSV>        CSV file with transactions to import
//!   -f, --format <FORMAT>  Date format
//!   -h, --help             Print help
//! ```
//!
//! To import transactions from a csv file some pre-formatting needs to be done
//! to ensure the columns names are mapped with the same names as in the toml files.
//!
//! Some banks provide statements in credit/debit format and other with sign or
//! parenthesis for negative values, so some number formatting may also be required.
//!
//! Running the import requires specifying the file where the toml transaction
//! are to be imported and the csv file with the details.
//!
//! Example of running the import script.
//!
//! ```bash
//! > abacus-rs -l example/transactions.toml import -c ~/Downloads/bbva/visa/sep23.csv
//! Import start
//! Imported: CsvRow { date: "2023-09-28", account: "Taxes", ...
//! Imported: CsvRow { date: "2023-09-23", account: "Dining",...
//! Imported: CsvRow { date: "2023-09-23", account: "Medical ...
//! ...
//! Import complete
//! ```

use clap::{Parser, Subcommand};
use csvimporter::import_transactions;
use ledger::Ledger;
use std::error::Error;
use utils::read_ledger_files;

pub mod accounts;
pub mod csvimporter;
pub mod ledger;
pub mod price;
pub mod transaction;
pub mod utils;

#[derive(Parser, Debug)]
#[command(author = "Federico Carles", version = "0.1", about, long_about = None)]
pub struct Args {
    /// Path to ledger file or directory
    #[arg(short, long)]
    ledger: String,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
    /// List accounts
    Accounts {},
    /// Print account balance sheet report
    Balances {
        /// Filter accounts by account type
        #[arg(short, long, num_args(0..))]
        class: Option<Vec<String>>,
        /// Filter transactions by start date
        #[arg(short, long)]
        from: Option<String>,
        /// Filter transactions by end date
        #[arg(short, long)]
        to: Option<String>,
        /// Price balances at specific currency
        #[arg(short, long)]
        price: Option<String>,
        /// Group balances by period (M, Q or Y)
        #[arg(short, long)]
        group: Option<String>,
    },
    /// Print transactions journal report
    Journal {
        /// Filter transactions by start date
        #[arg(short, long)]
        from: Option<String>,
        /// Filter transactions by end date
        #[arg(short, long)]
        to: Option<String>,
        /// Filter accounts by account type
        #[arg(short, long)]
        class: Option<String>,
        /// Filter accounts by account name
        #[arg(short, long)]
        account: Option<String>,
        /// Filter transactions by payee
        #[arg(short, long)]
        payee: Option<String>,
    },
    /// Import transactions from csv
    Import {
        /// CSV file with transactions to import
        #[arg(short, long)]
        csv: String,
        /// Date format
        #[arg(short, long)]
        format: Option<String>,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let ledger = read_ledger_files(&args.ledger);

    match args.command {
        Some(Commands::Accounts {}) => ledger?.print_accounts(),
        Some(Commands::Balances {
            from,
            to,
            class,
            price,
            group,
        }) => ledger?.print_balances(from, to, class, price, group),
        Some(Commands::Journal {
            from,
            to,
            class,
            account,
            payee,
        }) => ledger?.print_journal(from, to, class, account, payee),
        Some(Commands::Import { csv, format }) => import_transactions(&csv, &args.ledger, format)?,
        None => {}
    }
    Ok(())
}
