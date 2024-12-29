use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Sets the level of tracing.
    #[arg(short, long, global = true)]
    pub trace: Option<TraceLevel>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Webscrape data and collect it to the PostgreSQL database (findump).
    Spider {
        /// Specify the endpoints to webscrape.
        ///
        /// If no endpoints are provided, spider will collect all.
        #[arg(short, long)]
        endpoints: Option<Vec<Endpoint>>,
    },

    /// Test suite.
    Test,
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
#[clap(rename_all = "UPPERCASE")]
pub enum TraceLevel {
    DEBUG,
    ERROR,
    INFO,
    TRACE,
    WARN,
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum Endpoint {
    /// Cryptocurrency price data.
    Crypto,

    /// Economic data.
    Econ,

    /// Stock price & filings data.
    Stocks,
}
