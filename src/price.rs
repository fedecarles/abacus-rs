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
