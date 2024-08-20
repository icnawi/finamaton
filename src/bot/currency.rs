use reqwest;
use reqwest::header;
use serde_json::Value;
use std::error::Error;

use super::constants::currency::{CURRENCIES, CURRENCY_DEFAULT};

// Map of 3-letter currency code and number of decimals
pub type Currency = (String, i32);

// Converts str slice of currency code and decimals to Currency
fn to_currency(currency: (&str, i32)) -> Currency {
    (currency.0.to_string(), currency.1)
}

pub fn get_currency_from_code(code: &str) -> Option<Currency> {
    let code = code.to_uppercase();
    for currency in &CURRENCIES {
        if currency.0 == code {
            return Some(to_currency(*currency));
        }
    }

    None
}

pub fn get_default_currency() -> Currency {
    to_currency(CURRENCY_DEFAULT)
}

// TODO: Refactor this f
pub fn convert_currency(amount: i64, from: &str, to: &str, rate: f64) -> i64 {
    let from = match get_currency_from_code(from) {
        Some(currency) => currency,
        None => return amount,
    };

    let to = match get_currency_from_code(to) {
        Some(currency) => currency,
        None => return amount,
    };

    let result = amount as f64 * 10.0_f64.powi(to.1 - from.1) * rate;

    result.round() as i64
}

// Main API method that fetches currency conversions
pub async fn fetch_currency_conversion(from: &str, to: &str) -> Result<f64, Box<dyn Error>> {
    let from = from.to_lowercase();
    let to = to.to_lowercase();
    let url = format!(
        "https://cdn.jsdelivr.net/npm/@fawazahmed0/currency-api@latest/v1/currencies/{from}.json"
    );

    let mut h = header::HeaderMap::new();
    h.insert(
        "Accept",
        header::HeaderValue::from_static("application/json"),
    );

    let client = reqwest::Client::builder().default_headers(h).build()?;

    let response: Value = client.get(url).send().await?.json().await?;
    if let Some(conversions) = response.get(from) {
        if let Some(value) = conversions.get(to) {
            let res = value.as_f64();
            match res {
                Some(v) => return Ok(v),
                None => {
                    let res = value.as_i64();
                    if let Some(v) = res {
                        return Ok(v as f64);
                    }
                }
            }
        }
    }

    Err("Currency not found".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_currencies_api() {
        let fetch = fetch_currency_conversion("usd", "eur").await;
        assert!(fetch.is_ok());
        assert!(fetch.unwrap() > 0.0);
    }

    #[tokio::test]
    async fn test_fetch_non_existent_currencies() {
        let fetch = fetch_currency_conversion("usd", "non_existent_currency").await;
        assert!(fetch.is_err());
    }
}
