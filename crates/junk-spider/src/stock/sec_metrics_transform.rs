use super::common::Ticker;
use super::sec_metrics::Metric;
use chrono::NaiveDate;
use ordered_float::OrderedFloat;
use std::collections::{HashMap, HashSet};

/// The following process takes all Annual & Quarterly recordings, and insinuates any missing
/// Quarterly datapoints.
///
/// Tree {
///     (2023-01-01, 2023-12-31): {
///         metric_pk(1): {
///             (Annual Metric, Set<Metric>)
///         },
///         metric_pk(2): {
///             (Annual Metric, Set<Metric>)
///         }
///     },
/// }
#[derive(Debug)]
pub struct Insinuator {
    tree: HashMap<DateRange, HashMap<i32, (Metric, HashSet<Metric>)>>,
}

/// A range of chrono::NaiveDates.
///
/// DateRange {
///    start_date: NaiveDate,
///    end_date: NaiveDate,
/// }
#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct DateRange {
    start_date: NaiveDate,
    end_date: NaiveDate,
}

impl Insinuator {
    /// Build an Insinuator by pulling PostgreSQL queries and organising them by their `DateRange`.
    pub async fn new(pool: &deadpool_postgres::Pool, ticker: &Ticker) -> anyhow::Result<()> {
        let pg_client = pool.get().await.expect("failed to get pg client from pool");

        // initialise the tree with all the annual Metrics and their date ranges
        let mut tree: HashMap<DateRange, (Metric, HashSet<Metric>)> = HashMap::new();
        let _ = pg_client
            .query(
                "
                SELECT * FROM stock.metrics
                WHERE 
                    symbol_pk = $1 
                    AND
                    form = $2 
                    AND
                    start_date IS NOT NULL
                ",
                &[&ticker.pk, &"10-K"],
            )
            .await?
            .iter()
            .map(|row| {
                let metric = Metric {
                    symbol_pk: row.get(0),
                    metric_pk: row.get(1),
                    acc_pk: row.get(2),
                    start_date: row.get(3),
                    end_date: row.get(4),
                    filing_date: row.get(5),
                    year: row.get(6),
                    period: row.get(7),
                    form: row.get(8),
                    val: OrderedFloat(row.get(9)),
                    accn: row.get(10),
                    frame: row.get(11),
                };
                tree.insert(
                    DateRange {
                        start_date: metric.start_date.expect("failed to get start_date"),
                        end_date: metric.end_date,
                    },
                    (metric, HashSet::new()),
                );
            })
            .collect::<Vec<_>>();

        println!("{tree:#?}");

        Ok(())
    }
}
