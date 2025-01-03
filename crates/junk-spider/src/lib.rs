//////////////////////////////////////////////////////////////////////
///
/// Data collection libraries
///
//////////////////////////////////////////////////////////////////////

/// Cryptocurrency data, collected from the REST APIs of various exchanges.
///
/// Examples include **Binance, KuCoin, MEXC, Kraken**.
pub mod crypto;

/// Economic data;
///
/// - US data collected from [FRED](https://fred.stlouisfed.org/docs/api/fred/).
pub mod econ;

/// Stock data, collected from various sources.
///
/// Examples include **Yahoo! Finance & the SEC**.
pub mod stock;

//////////////////////////////////////////////////////////////////////
///
/// Utilities
///
//////////////////////////////////////////////////////////////////////

/// Colored logging function.
pub(crate) fn time_elapsed(time: std::time::Instant) -> String {
    use colored::Colorize;
    format!("< Time elapsed: {} ms >", time.elapsed().as_millis())
        .truecolor(224, 60, 138)
        .to_string()
}

/// Standard client build for HTTP requests, only requiring a User-Agent Environrment Variable.
pub(crate) fn std_client_build() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(&dotenv::var("USER_AGENT").expect("failed to read USER_AGENT"))
        .build()
        .expect("failed to build reqwest::Client")
}

/// Shortcuts used in HTTP API requests.
pub mod http {
    pub use dotenv::var;
    pub use reqwest::Client as HttpClient;
    pub use tokio_postgres::Client as PgClient;
}

/// File store functions.
pub mod fs;

/// Common TUI functions.
pub mod tui {
    use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
    use std::time::Duration;

    pub(crate) fn multi_progress(
        len: usize,
    ) -> anyhow::Result<(
        Option<MultiProgress>,
        Option<ProgressBar>,
        Option<ProgressBar>,
        Option<ProgressBar>,
    )> {
        // overall multi progress bar
        let multi = MultiProgress::new();

        // total number of tickers to collect
        let total = multi.add(
            ProgressBar::new(len as u64).with_style(
                ProgressStyle::default_bar()
                    .template(
                        "collecting crypto prices ... {spinner:.magenta}\n \
                        {msg:>9.white} |{bar:57.white/grey}| {pos:<2} / {human_len} \
                        ({percent_precise}%) [Time: {elapsed}, Rate: {per_sec}, ETA: {eta}]",
                    )?
                    .progress_chars("## "),
            ),
        );
        total.set_message("total");
        total.enable_steady_tick(Duration::from_millis(100));

        // total successful collections
        let success = multi.insert_after(
            &total,
            ProgressBar::new(len as u64).with_style(
                ProgressStyle::default_bar()
                    .template(" {msg:>9.green} |{bar:57.green}| {pos:<2.green}")?
                    .progress_chars("## "),
            ),
        );
        success.set_message("successes");

        // total failed collections
        let fails = multi.insert_after(
            &success,
            ProgressBar::new(len as u64).with_style(
                ProgressStyle::default_bar()
                    .template(" {msg:>9.red} |{bar:57.red}| {pos:<2.red}")?
                    .progress_chars("## "),
            ),
        );
        fails.set_message("failures");

        Ok((Some(multi), Some(total), Some(success), Some(fails)))
    }
}
