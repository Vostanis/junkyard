use deadpool_postgres::Pool;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Deserializer};
use tracing::{error, trace};

/// Used within the SEC datasets; each company is given a CIK code (and ticker, and title),
/// intended to be a 10-character string.
pub fn de_cik<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    // general deserialisation, followed by match statement (depending on type found)
    let value: serde_json::Value = Deserialize::deserialize(deserializer)?;
    match value {
        // if it's a num, pad it as a 10-char string
        serde_json::Value::Number(num) => {
            if let Some(i32_value) = num.as_i64() {
                // as_i64() does the same job for i32
                Ok(format!("{:010}", i32_value))
            } else {
                Err(serde::de::Error::custom(
                    "ERROR! Unable to parse i32 from JSON",
                ))
            }
        }

        // if it's a string, then Ok()
        serde_json::Value::String(s) => Ok(s),

        // else return an error (it can't be correct type)
        _ => Err(serde::de::Error::custom("ERROR! Invalid type for CIK")),
    }
}

/// Convert a &String to a chrono::NaiveDate (so that it can inserted directly as DATE)
pub fn convert_date_type(str_date: &String) -> anyhow::Result<chrono::NaiveDate> {
    let date = chrono::NaiveDate::parse_from_str(&str_date, "%Y-%m-%d").map_err(|err| {
        error!("failed to parse date string; expected form YYYYMMDD - received: {str_date}, error({err})");
        err
    })?;
    Ok(date)
}

/// Convert a u32 timestamp to a chrono::NaiveDate.
pub fn convert_timestamp(timestamp: u32) -> chrono::NaiveDate {
    chrono::DateTime::from_timestamp(timestamp.into(), 0)
        .expect("Expected Vector of Timestamp integers")
        .date_naive()
}

/// List of all tickers in the database.
pub struct Tickers(pub Vec<Ticker>);

/// Individual ticker information.
#[derive(Clone, Debug, Deserialize)]
pub struct Ticker {
    pub pk: i32,
    pub file_code: String,
    pub ticker: String,
    pub title: String,
}

impl Tickers {
    /// Fetch all tickers from the database.
    pub async fn fetch_tickers(pool: &Pool) -> anyhow::Result<Self> {
        // wait for a pg client from the pool
        let pg_client = pool.get().await.expect("failed to get pg client from pool");

        // return all tickers from the database
        trace!("fetching tickers ...");
        let tickers: Vec<Ticker> = pg_client
            .query(
                "SELECT pk, file_code, symbol, title FROM stock.symbols",
                &[],
            )
            .await
            .expect("failed to fetch stock.tickers")
            .into_par_iter()
            .map(|row| Ticker {
                pk: row.get(0),
                file_code: row.get(1),
                ticker: row.get(2),
                title: row.get(3),
            })
            .collect();

        Ok(Tickers(tickers))
    }
}
