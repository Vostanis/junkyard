mod cli;

use clap::Parser;
use cli::{Cli, TraceLevel};
use tracing::{debug, error, info, subscriber, trace, Level};
use tracing_subscriber::FmtSubscriber;

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
            // 2. match pre-built endpoints to scrape
        }

        // test env
        cli::Commands::Test => {
            println!("test command");
        } // cli::Commands::Yard => {
          //     tui
          // }
    }

    Ok(())
}
