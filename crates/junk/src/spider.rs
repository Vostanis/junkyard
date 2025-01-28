use crate::cli::Endpoint;
use deadpool_postgres::{ManagerConfig, RecyclingMethod};
use dotenv::var;
// use junk_spider as spider;
use tracing::{debug, info, trace};

/// Run all working spider processes.
pub(crate) async fn run(endpoints: Vec<Endpoint>, tui: bool) -> anyhow::Result<()> {
    // 1. build pg pool connection
    trace!("creating postgres connection pool config");
    let mut pg_config = deadpool_postgres::Config::new();
    pg_config.url = Some(var("FINDUMP_URL")?);
    pg_config.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    trace!("creating findump connection pool");
    let pool = pg_config.create_pool(
        Some(deadpool_postgres::Runtime::Tokio1),
        tokio_postgres::NoTls,
    )?;
    debug!("findump connection pool established");

    // start collecting data
    let time = std::time::Instant::now();
    for endpoint in endpoints {
        match endpoint {
            Endpoint::Crypto => {
                use junk_spider::crypto;

                let time = std::time::Instant::now();

                crypto::mexc::scrape(&pool, tui).await?;
                crypto::kraken::scrape(&pool, tui).await?;
                crypto::binance::scrape(&pool, tui).await?;
                crypto::kucoin::scrape(&pool, tui).await?;

                info!("crypto data collected, time elapsed: {:?}", time.elapsed());
            }

            Endpoint::Econ => {
                // use junk_spider::econ;
                let time = std::time::Instant::now();

                // econ::fred::scrape(&pool).await?;

                info!(
                    "economic data collected, time elapsed: {:?}",
                    time.elapsed()
                );
            }

            Endpoint::Stocks => {
                use junk_spider::stock;
                let time = std::time::Instant::now();

                // stock::sec_bulks::scrape(tui).await?;
                stock::sec_tickers::scrape(&pool, tui).await?;
                stock::yahoo_finance::scrape(&pool, tui).await?;
                // stock::sec_metrics::scrape(&pool, tui).await?;

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
