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
    let log_level = match cli.trace {
        TraceLevel::DEBUG => Level::DEBUG,
        TraceLevel::ERROR => Level::ERROR,
        TraceLevel::INFO => Level::INFO,
        TraceLevel::TRACE => Level::TRACE,
        TraceLevel::WARN => Level::WARN,
    };
    preprocess(log_level);
    trace!("command line input recorded: {cli:?}");

    // read cli inputs
    use cli::Commands::*;
    match cli.command {
        // scrape endpoints
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

            // 2. match pre-built endpoints to scrape
            spider::crypto::binance::scrape(&mut pg_client).await?;

            // match subcommand somehow
        }

        // test env
        Test => {
            println!("Hello, World!");
        }
    }

    Ok(())
}
