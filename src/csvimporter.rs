//! This module is the csv transaction importer.

use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write;
use toml::to_string_pretty;

#[derive(Debug, Deserialize, Serialize)]
struct CsvRow {
    #[serde(rename = "date")]
    date_str: String,
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
        println!("{}", row.date_str);
        row.date_str = row.date_str.to_owned();
        let date_format = date_format.as_deref().unwrap_or("%d/%m/%Y");
        let date = NaiveDate::parse_from_str(&row.date_str, date_format)?;
        row.date_str = date.to_string();
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

    println!("Import start");
    for t in new_transactions {
        println!("Imported: {:?}", t);
        let toml_value = to_string_pretty(&t)?;
        file.write("\n[[transaction]]\n".as_bytes())?;
        file.write_all(toml_value.as_bytes())
            .expect("Failed to write to file");
    }
    println!("Import complete");

    Ok(())
}
