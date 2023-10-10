use crate::transaction::Transaction;
use crate::utils::deserialize_date;
use chrono::prelude::*;
use csv::Reader;
use serde_derive::{Deserialize, Serialize};
use std::fs::{read_to_string, File, OpenOptions};
use std::{error::Error, io, io::Write, process};
use toml::{to_string_pretty, Value};

#[derive(Debug, Deserialize, Serialize)]
struct CsvRow {
    date: String,
    account: String,
    payee: Option<String>,
    quantity: Option<f32>,
    amount: f32,
    offset_account: Option<String>,
    offset_amount: Option<f32>,
}
pub fn import_transactions(
    csv_file: &str,
    toml_file: &str,
    date_format: Option<String>,
) -> Result<(), Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path(csv_file)?;

    let mut new_transactions: Vec<CsvRow> = vec![];

    for result in rdr.deserialize() {
        let mut row: CsvRow = result?;
        row.date = NaiveDate::parse_from_str(
            row.date.as_str(),
            &date_format.clone().unwrap_or("%d/%m/%Y".to_string()),
        )
        .unwrap_or_default()
        .to_string();
        row.amount = row.amount.abs();
        row.quantity = None;
        row.offset_account = row.offset_account;
        row.offset_amount = Some(row.amount * -1.0);

        new_transactions.push(row);
    }

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(toml_file)?;

    for t in new_transactions {
        let toml_value = to_string_pretty(&t)?;
        file.write("\n[[transaction]]\n".as_bytes());
        file.write_all(toml_value.to_string().as_bytes())
            .expect("Failed to write to file");
    }
    Ok(())
}
