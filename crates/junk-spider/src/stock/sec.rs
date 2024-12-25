use crate::http::*;
use serde::Deserialize;

// tickers
async fn scrape_tickers() -> anyhow::Result<()> {
    let client = HttpClient::new();
    let response = client
        .get("https://www.sec.gov/files/company_tickerrs.json")
        .send()
        .await?;
    let body = response.json().await?;
    Ok(())
}

#[derive(Debug, Deserialize)]
struct Tickers {}

// metrics

// filings
