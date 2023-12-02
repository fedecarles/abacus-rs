//! This module defines the commodity [Price] struct.
//!
//! Declaring Commodity prices is entirely optional but very useful to price
//! stocks or currencies.
//!
//! ```toml
//! [[price]]
//! date = 2023-10-02
//! commodity = "ARS"
//! price = 0.00125
//! currency = "USD"
//!
//! [[price]]
//! date = 2023-09-30
//! commodity = "VOO"
//! price = 390.50
//! currency = "USD"
//! ```

use chrono::prelude::*;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Price {
    pub date: NaiveDate,
    pub commodity: String,
    pub price: f32,
    pub currency: String,
}

impl Price {
    pub fn new(date: NaiveDate, commodity: String, price: f32, currency: String) -> Self {
        Self {
            date: date,
            commodity: commodity.replace('"', ""),
            price: price,
            currency: currency.replace('"', ""),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_new() {
        let date = NaiveDate::from_ymd_opt(2023, 10, 13).unwrap();
        let commodity = "Gold".to_string();
        let price = 1500.0;
        let currency = "USD".to_string();

        let price_data = Price::new(date, commodity.clone(), price, currency.clone());

        assert_eq!(price_data.date, date);
        assert_eq!(price_data.commodity, commodity);
        assert_eq!(price_data.price, price);
        assert_eq!(price_data.currency, currency);
    }

    #[test]
    fn test_price_equality() {
        let date1 = NaiveDate::from_ymd_opt(2023, 10, 13).unwrap();
        let commodity1 = "Gold".to_string();
        let price1 = 1500.0;
        let currency1 = "USD".to_string();

        let date2 = NaiveDate::from_ymd_opt(2023, 10, 14).unwrap();
        let commodity2 = "Gold".to_string();
        let price2 = 1550.0;
        let currency2 = "USD".to_string();

        let price_data1 = Price::new(date1, commodity1.clone(), price1, currency1.clone());
        let price_data2 = Price::new(date2, commodity2.clone(), price2, currency2.clone());

        // Prices with different dates should not be equal
        assert_ne!(price_data1, price_data2);

        // Prices with the same date, commodity, price, and currency should be equal
        let price_data3 = Price::new(date1, commodity1, price1, currency1);
        assert_eq!(price_data1, price_data3);
    }

    #[test]
    fn test_price_clone() {
        let date = NaiveDate::from_ymd_opt(2023, 10, 13).unwrap();
        let commodity = "Gold".to_string();
        let price = 1500.0;
        let currency = "USD".to_string();

        let price_data = Price::new(date, commodity.clone(), price, currency.clone());
        let cloned_price_data = price_data.clone();

        assert_eq!(price_data, cloned_price_data);
    }
}
