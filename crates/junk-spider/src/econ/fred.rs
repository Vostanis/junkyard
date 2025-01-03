use super::sql;
use crate::http::*;
use crate::stock::common::convert_date_type;
use deadpool_postgres::Pool;
use futures::{stream, StreamExt};
use serde::Deserialize;
use std::sync::Arc;
use tracing::{debug, error, trace};

pub async fn scrape(pool: &Pool) -> anyhow::Result<()> {
    let http_client = crate::std_client_build();

    for dataset in ["Unemployment Rate", "Interest Rate"] {
        let key = &var("FRED_API").expect("environment variable FRED_API");
        let url = match dataset {
                "Unemployment Rate" => 
                    format!("https://api.stlouisfed.org/fred/series/observations?series_id=DFF&api_key={key}&file_type=json"),
                "Interest Rate" => 
                    format!("https://api.stlouisfed.org/fred/series/observations?series_id=UNRATE&api_key={key}&file_type=json"),
                _ => panic!("unexpected FRED dataset"),
            };

        trace!("fetching Fred data {dataset}");
        let data: Observations = http_client.get(url).send().await?.json().await?;

        // wait for pg client
        let mut pg_client = pool.get().await?;

        trace!("inserting Fred data {dataset}");
        data.insert(&mut pg_client, &dataset).await?;
    }

    Ok(())
}

//////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize)]
struct Observations {
    #[serde(rename = "observations")]
    inner: Vec<Observation>,
}

#[derive(Debug, Deserialize)]
struct Observation {
    #[serde(rename = "date")]
    dated: String,
    value: String,
}

impl Observations {
    async fn insert(&self, pg_client: &mut PgClient, metric: &str) -> anyhow::Result<()> {
        let time = std::time::Instant::now();

        // open sql TRANSACTION
        let query = pg_client.prepare(sql::INSERT_METRIC).await?;
        let transaction = Arc::new(pg_client.transaction().await?);

        // async stream insert into TRANSACTION
        let mut stream = stream::iter(&self.inner);
        while let Some(cell) = stream.next().await {
            let query = &query;
            let transaction = transaction.clone();
            async move {
                let dated = convert_date_type(&cell.dated).expect("error converting date type");
                let val = cell.value.parse::<f64>().expect("error parsing value");
                let result = transaction.execute(query, &[&dated, &metric, &val]).await;

                match result {
                    Ok(_) => trace!("inserted US interest rate [{dated}, {metric}, {val}]"),
                    Err(e) => error!("error inserting interest rate: {:?}", e),
                }
            }
            .await;
        }

        match Arc::into_inner(transaction)
            .expect("failed to unpack Transaction from Arc")
            .commit()
            .await
        {
            Ok(_) => trace!("committed transaction for econ.us"),
            Err(e) => error!("error committing transaction for econ.us: {:?}", e),
        }

        debug!("inserted FRED.{metric} {}", crate::time_elapsed(time));

        Ok(())
    }
}
