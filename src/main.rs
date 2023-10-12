// #![allow(warnings)]
use clap::{Parser, Subcommand};
use csvimporter::import_transactions;
use ledger::Ledger;
use std::error::Error;
//use std::io::Error;
use utils::read_ledger_files;

mod accounts;
mod csvimporter;
mod ledger;
mod price;
mod transaction;
mod utils;

#[derive(Parser, Debug)]
#[command(author = "Federico Carles", version = "0.1", about, long_about = None)]
struct Args {
    /// Path to ledger file or directory
    #[arg(short, long)]
    ledger: String,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Clone, Subcommand)]
enum Commands {
    /// List accounts
    Accounts {},
    /// Print account balance sheet report
    Balances {
        /// Filter accounts by account type
        #[arg(short, long, num_args(0..))]
        class: Option<Vec<String>>,
        /// Filter transactions by year
        #[arg(short, long)]
        year: Option<String>,
        /// Price balances at specific currency
        #[arg(short, long)]
        price: Option<String>,
    },
    /// Print transactions journal report
    Journal {
        /// Filter transactions by year
        #[arg(short, long)]
        year: Option<String>,
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
        Some(Commands::Balances { year, class, price }) => {
            ledger?.print_balances(year, class, price)
        }
        Some(Commands::Journal {
            year,
            class,
            account,
            payee,
        }) => ledger?.print_journal(year, class, account, payee),
        Some(Commands::Import { csv, format }) => import_transactions(&csv, &args.ledger, format)?,
        None => {}
    }
    Ok(())
}
