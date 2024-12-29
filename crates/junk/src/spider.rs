use crate::cli::Endpoint;
use dotenv::var;
use junk_spider as spider;
use tokio_postgres::{self as pg, NoTls};
use tracing::{debug, error, info, trace};

/// Run all working spider processes.
pub(crate) async fn run(endpoints: Vec<Endpoint>) -> anyhow::Result<()> {
    // 1. build pg connection
    trace!("connecting to findump ...");
    let (mut pg_client, pg_conn) = pg::connect(
        &var("FINDUMP_URL").expect("environment variable FINDUMP_URL"),
        NoTls,
    )
    .await
    .map_err(|err| {
        error!("findump connection error: {}", err);
        err
    })?;

    tokio::spawn(async move {
        if let Err(err) = pg_conn.await {
            error!("findump connection error: {}", err);
        }
    });
    debug!("findump connection established");

    // start collecting data
    let time = std::time::Instant::now();
    for endpoint in endpoints {
        match endpoint {
            Endpoint::Crypto => {
                let time = std::time::Instant::now();

                spider::crypto::binance::scrape(&mut pg_client).await?;
                spider::crypto::kucoin::scrape(&mut pg_client).await?;
                spider::crypto::mexc::scrape(&mut pg_client).await?;
                spider::crypto::kraken::scrape(&mut pg_client).await?;

                info!("crypto data collected, time elapsed: {:?}", time.elapsed());
            }
            Endpoint::Econ => {
                let time = std::time::Instant::now();

                spider::econ::fred::scrape(&mut pg_client).await?;

                info!(
                    "economic data collected, time elapsed: {:?}",
                    time.elapsed()
                );
            }
            Endpoint::Stocks => {
                let time = std::time::Instant::now();

                spider::stock::sec::bulks::scrape().await?;
                spider::stock::sec::tickers::scrape(&mut pg_client).await?;
                spider::stock::sec::metrics::scrape(&mut pg_client).await?;
                spider::stock::yahoo_finance::scrape(&mut pg_client).await?;

                info!("stock data collected, time elapsed: {:?}", time.elapsed());
            }
        }
    }

    info!(
        "spider finished collecting data, time elapsed: {:?}",
        time.elapsed()
    );

    Ok(())
}
