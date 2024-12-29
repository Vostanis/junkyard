mod cli;

// remote imports
use clap::Parser;
use cli::{Cli, TraceLevel};
use dotenv::var;
use tokio_postgres::{self as pg, NoTls};
use tracing::{debug, error, info, subscriber, trace, Level};
use tracing_subscriber::FmtSubscriber;

// preproccess the trace level
fn preprocess(trace_level: Level) {
    dotenv::dotenv().ok();
    let my_subscriber = FmtSubscriber::builder()
        .with_max_level(trace_level)
        .finish();
    subscriber::set_global_default(my_subscriber).expect("Set subscriber");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // set the trace level
    if let Some(trace_level) = cli.trace {
        preprocess(match trace_level {
            TraceLevel::DEBUG => Level::DEBUG,
            TraceLevel::ERROR => Level::ERROR,
            TraceLevel::INFO => Level::INFO,
            TraceLevel::TRACE => Level::TRACE,
            TraceLevel::WARN => Level::WARN,
        });
    }
    trace!("command line input recorded: {cli:?}");

    // read cli inputs
    use cli::Commands::*;
    match cli.command {
        // `junk spider <subarg>`: scrape endpoints
        Spider => {
            use junk_spider as spider;

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

            // start the clock
            let time = std::time::Instant::now();

            // // 2. download crypto
            // spider::crypto::binance::scrape(&mut pg_client).await?;
            // spider::crypto::kucoin::scrape(&mut pg_client).await?;
            // spider::crypto::mexc::scrape(&mut pg_client).await?;
            // spider::crypto::kraken::scrape(&mut pg_client).await?;
            //
            // info!(
            //     "crypto finishing scraping, time elapsed: {:?}",
            //     time.elapsed()
            // );

            // 3. download stocks
            //    a) download bulks
            // spider::stock::sec::bulks::scrape().await?;
            // spider::stock::sec::tickers::scrape(&mut pg_client).await?;
            // spider::stock::sec::metrics::scrape(&mut pg_client).await?;
            // spider::stock::yahoo_finance::scrape(&mut pg_client).await?;

            // 4. download economic data
            spider::econ::fred::scrape(&mut pg_client).await?;

            info!(
                "crypto finishing scraping, time elapsed: {:?}",
                time.elapsed()
            );
        }

        // test env
        Test => {
            println!("Hello, World!");
        }
    }

    Ok(())
}
