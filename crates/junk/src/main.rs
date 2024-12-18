mod cli;

// remote imports
use clap::Parser;
use cli::{Cli, TraceLevel};
use dotenv::var;
use tokio_postgres::{self as pg, NoTls};
use tracing::{debug, error, info, subscriber, trace, Level};
use tracing_subscriber::FmtSubscriber;

// local modules
use junk_spider as spider;

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
    match cli.command {
        // scrape endpoints
        cli::Commands::Spider => {
            // 1. build pg connection
            trace!("Establishing PostgreSQL connection");
            let (mut pg_client, pg_conn) = pg::connect(
                &var("POSTGRES_URL").expect("environment variable POSTGRES_URL"),
                NoTls,
            )
            .await?;
            tokio::spawn(async move {
                if let Err(e) = pg_conn.await {
                    error!("connection error: {}", e);
                }
            });
            debug!("PostgreSQL connection established");

            // 2. match pre-built endpoints to scrape
            spider::crypto::binance::scrape(&mut pg_client).await?;

            // match subcommand somehow
        }

        // test env
        cli::Commands::Test => {
            println!("Hello, World!");
        }
    }

    Ok(())
}
