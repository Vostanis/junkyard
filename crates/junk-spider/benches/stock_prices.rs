use criterion::Criterion;
use serde::Deserialize;

// util
fn read_file_to_string(path: &str) -> String {
    let mut file = File::open(path).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");
    contents
}

// first approach
// ====================
//
// 1. standard deserialization
// 2. transform to vec
fn default() {}

// second approach
// ====================
//
// deserialization

// schemas
// ====================

// 1. default deserialization
#[derive(Debug, Deserialize)]
struct PriceResponse {
    chart: Chart,
}

#[derive(Debug, Deserialize)]
struct Chart {
    result: Option<Vec<Result>>,
}

#[derive(Debug, Deserialize)]
struct Result {
    timestamp: Vec<i64>,
    indicators: Indicators,
}

#[derive(Debug, Deserialize)]
struct Indicators {
    quote: Vec<Quote>,
    adjclose: Vec<AdjClose>,
}

#[derive(Debug, Deserialize)]
struct Quote {
    open: Vec<f64>,
    high: Vec<f64>,
    low: Vec<f64>,
    close: Vec<f64>,
    volume: Vec<i64>,
}

#[derive(Debug, Deserialize)]
struct AdjClose {
    adjclose: Vec<f64>,
}
