//! This module contains utility functions.

use crate::Ledger;
use chrono::prelude::*;
use serde::{Deserialize, Deserializer};
use std::error::Error;
use std::fs::{self, read_to_string};
use std::str::FromStr;
use toml::Value;

/// Reads a single toml file from a file path or multiple toml files from
/// a directory.
pub fn read_ledger_files(ledger_path: &str) -> Result<Ledger, Box<dyn Error>> {
    let ledger = match fs::metadata(ledger_path) {
        Ok(file) => {
            if file.is_dir() {
                let mut concatenated_files = String::new();
                let files = fs::read_dir(ledger_path)?;
                for f in files {
                    let f = f?;
                    let file_path = f.path();

                    if file_path.is_file() && file_path.extension().unwrap_or_default() == "toml" {
                        let toml_content = read_to_string(file_path).expect("Failed to read file.");
                        concatenated_files.push_str(&toml_content);
                    }
                }
                if concatenated_files.is_empty() {
                    return Err("No TOML files found in directory".into());
                }
                Ledger::new(&concatenated_files)
            } else {
                let toml_content = fs::read_to_string(ledger_path).expect("Failed to read file.");
                Ledger::new(&toml_content)
            }
        }
        Err(_) => Ledger::new(ledger_path),
    };
    return ledger;
}

/// Deserialize a NaiveDate from a string
pub fn deserialize_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let date_str = String::deserialize(deserializer)?;
    let formats = ["%d/%m/%Y", "%Y-%m-%d"];
    for format in &formats {
        if let Ok(date) = NaiveDate::parse_from_str(&date_str, format) {
            return Ok(date);
        }
    }
    Err(serde::de::Error::custom("Invalid date format"))
}

/// Parse toml values to f32.
pub fn parse_value_to_f32<T>(value: &Value, key: &str) -> Option<f32> {
    let toml_value = value.get(key);
    let float_value = match toml_value.unwrap_or(&Value::Float(0.0)) {
        Value::Integer(integer_value) => *integer_value as f32,
        Value::Float(float_value) => *float_value as f32,
        _ => panic!("Amount is not an integer or float"),
    };
    return Some(float_value);
}

/// Parse toml values to NaiveDate.
pub fn parse_value_to_naivedate(val: &Value, col: &str) -> Option<NaiveDate> {
    let date_val = val.get(col)?;
    if let Some(s) = date_val.as_str() {
        return NaiveDate::from_str(s).ok();
    }
    if let Some(dt) = date_val.as_datetime() {
        if let Some(d) = dt.date {
            return NaiveDate::from_str(&d.to_string()).ok();
        }
    }
    None
}

/// Parse any string toml value.
pub fn parse_value<T>(value: &Value, key: &str) -> Option<T>
where
    T: FromStr,
{
    value
        .get(key)
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
}

/// Map months to quarters.
pub fn quarter(month: u32) -> u32 {
    match month {
        1 | 2 | 3 => 1,
        4 | 5 | 6 => 2,
        7 | 8 | 9 => 3,
        10 | 11 | 12 => 4,
        _ => unreachable!(),
    }
}
