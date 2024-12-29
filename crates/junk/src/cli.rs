use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Sets the level of tracing
    #[arg(long, global = true)]
    pub trace: Option<TraceLevel>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Spider,
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
