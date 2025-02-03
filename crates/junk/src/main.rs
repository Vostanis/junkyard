mod cli;
mod spider;

// remote imports
use crate::cli::Endpoint::*;
use clap::Parser;
use cli::{Cli, TraceLevel};
use deadpool_postgres::{ManagerConfig, RecyclingMethod};
use dotenv::var;
use tracing::{debug, subscriber, trace, Level};
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

    // if no trace level provided, use tui
    let tui = match cli.trace {
        Some(_) => false,
        None => true,
    };

    // read cli inputs
    use cli::Commands::*;
    match cli.command {
        // `junk spider <Option<Vec<Endpoint>>>`: scrape endpoints
        Spider { endpoints } => {
            // if no endpoints provided, scrape all
            match endpoints {
                Some(endpoints) => spider::run(endpoints, tui).await?,
                None => spider::run(vec![Crypto, Econ, Stocks], tui).await?,
            }
        }

        // test env
        Test => {
            // use junk_spider::stock::common::Ticker;
            // use junk_spider::*;
            //
            // trace!("creating postgres connection pool config");
            // let mut pg_config = deadpool_postgres::Config::new();
            // pg_config.url = Some(var("FINDUMP_URL")?);
            // pg_config.manager = Some(ManagerConfig {
            //     recycling_method: RecyclingMethod::Fast,
            // });
            //
            // trace!("creating findump connection pool");
            // let pool = pg_config.create_pool(
            //     Some(deadpool_postgres::Runtime::Tokio1),
            //     tokio_postgres::NoTls,
            // )?;
            // debug!("findump connection pool established");
            //
            // let ticker = Ticker {
            //     pk: 10546,
            //     ticker: String::from("NVDA"),
            //     title: String::from("NVIDIA Corp."),
            //     file_code: String::from("0001045810"),
            // };
            //
            // stock::sec_metrics_transform::Insinuator::new(&pool, &ticker).await?;
        }
    }

    Ok(())
}
