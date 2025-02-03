#![allow(dead_code)]
#![allow(unused_imports)]

use super::common::{Ticker, Tickers};
use super::sec_metrics::Metric;
use chrono::NaiveDate;
use futures::{stream, StreamExt};
use ordered_float::OrderedFloat;
use std::collections::{HashMap, HashSet};

/// ### Methodology
///
/// 1. Loop through the tickers.
///
/// 2. Loop through the 10-K metrics, per ticker.
///
/// 3. Has the metric been used yet? If no, create it.
///
///         HashMap<i32, AnnualMetric>
///
/// 4. Within the HashMap, there is the 10-K date ranges.
///    Does the date range exist? If no, create it.
///
///         HashMap<(NaiveDate, NaiveDate), HashSet<f64>>
struct Tree {
    inner: HashMap<i32, AnnualMetric>,
}

async fn run(pool: &deadpool_postgres::Pool) -> anyhow::Result<()> {
    let tickers = Tickers::fetch_tickers(pool).await?;

    stream::iter(tickers.0)
        .for_each_concurrent(num_cpus::get(), |stock| {
            // per ticker, build a tree, calculate missing values,
            // and then COPY them to the pg database
            async move {
                let mut tree = Tree::new();
                tree.fetch_annuals(stock, pool).await.unwrap();
                // tr.fetch_quarterlies(stock, tree).await?;
            }
        })
        .await;

    Ok(())
}

impl Tree {
    /// Create a new Tree.
    fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Insert the Annual metrics into the Tree.
    async fn fetch_annuals(
        &mut self,
        stock: Ticker,
        pool: &deadpool_postgres::Pool,
    ) -> anyhow::Result<()> {
        let pg_rows = pool
            .get()
            .await?
            .query(
                "SELECT * FROM stock.metrics 
                WHERE symbol_pk = $1
                AND form = $2",
                &[&stock.pk],
            )
            .await?;

        Ok(())
    }

    /// Sort and insert the Quarterly metrics into the Tree.
    async fn fetch_quarterlies(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
struct AnnualMetric {
    start_date: NaiveDate,
    end_date: NaiveDate,
    total: f64,
    values: HashSet<f64>,
}

impl AnnualMetric {
    /// Calculate the difference between the total and the sum of the values.
    fn calc(&self) -> f64 {
        self.total - self.values.iter().sum::<f64>()
    }

    /// Check if a metric is within the date range.
    fn check(&self, metric: &Metric) -> bool {
        // unwrap Option<NaiveDate>
        if let Some(start_date) = metric.start_date {
            start_date >= self.start_date && metric.end_date <= self.end_date
        } else {
            // if Option<NaiveDate> returns None, there has been an error in retrieving
            // from Postgres
            tracing::error!("'start_date' column found to be empty for {:?}", self);
            false
        }
    }
}
