use crate::http::*;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::ClientBuilder;

// RATE_LIMIT = 1200 /60s
//
// tickers = `https://api.binance.com/api/v1/ticker/allBookTickers`
//
// klines = `https://api.binance.com/api/v3/klines`, per symbol

/////////////////////////////////////////////////////////////////////////////////
// core
/////////////////////////////////////////////////////////////////////////////////

// binance http client requires "X-MBX-APIKEY"
fn build_client() -> HttpClient {
    let mut headers = HeaderMap::new();
    headers.insert(
        "X-MBX-APIKEY",
        HeaderValue::from_str(&var("BINANCE_API").expect("BINANCE_API not found"))
            .expect("failed to set BINANCE_API as X-MBX-APIKEY header"),
    );
    let client = ClientBuilder::new()
        .default_headers(headers)
        .build()
        .expect("Binance Client to build");
    client
}

/////////////////////////////////////////////////////////////////////////////////
// endpoints
/////////////////////////////////////////////////////////////////////////////////

// tickers

// prices
