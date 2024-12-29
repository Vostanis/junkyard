mod cli;
mod spider;

// remote imports
use crate::cli::Endpoint::*;
use clap::Parser;
use cli::{Cli, TraceLevel};
use tracing::{subscriber, trace, Level};
use tracing_subscriber::FmtSubscriber;

////////////////////////////////////////////////////////////////////////////

// preproccess the trace level, and open the .env file
fn preprocess(trace_level: Level) {
    dotenv::dotenv().ok();
    let my_subscriber = FmtSubscriber::builder()
        .with_max_level(trace_level)
        .finish();
    subscriber::set_global_default(my_subscriber).expect("Set subscriber");
}

////////////////////////////////////////////////////////////////////////////

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
        // `junk spider <Option<Vec<Endpoint>>>`: scrape endpoints
        Spider { endpoints } => {
            // if no endpoints provided, scrape all
            match endpoints {
                Some(endpoints) => spider::run(endpoints).await?,
                None => spider::run(vec![Crypto, Econ, Stocks]).await?,
            }
        }

        // test env
        Test => {
            trace!("connecting to findump ...");
            let (mut pg_client, pg_conn) = tokio_postgres::connect(
                &dotenv::var("FINDUMP_URL").expect("environment variable FINDUMP_URL"),
                tokio_postgres::NoTls,
            )
            .await
            .map_err(|err| {
                tracing::error!("findump connection error: {}", err);
                err
            })?;

            tokio::spawn(async move {
                if let Err(err) = pg_conn.await {
                    tracing::error!("findump connection error: {}", err);
                }
            });
            tracing::debug!("findump connection established");

            junk_spider::stock::yahoo_finance::scrape(&mut pg_client).await?;
        }
    }

    Ok(())
}
