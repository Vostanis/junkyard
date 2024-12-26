use crate::fs::{download_file, unzip};
use crate::http::*;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio::sync::Mutex;
use tracing::{debug, error, info, trace};

// 1. check if downloads are necessary
// 2. download if necessary
//     a. metrics
//     b. submissions
// 3. unzip the files; and delete the zips

const METRICS_URL: &'static str =
    "https://www.sec.gov/Archives/edgar/daily-index/xbrl/companyfacts.zip";

const SUBMISSIONS_URL: &'static str =
    "https://www.sec.gov/Archives/edgar/daily-index/bulkdata/submissions.zip";

/// Scrape the SEC website for the latest company metrics and filings metadata.
async fn scrape() -> anyhow::Result<()> {
    let http_client = build_client();

    info!("downloading companyfacts.zip and submissions.zip ...");

    // download companyfacts.zip (the metrics)
    debug!("downloading metrics.zip ...");
    download_file(&http_client, METRICS_URL, "./buffer/metrics.zip")
        .await
        .map_err(|err| {
            error!("failed to download metrics.zip: {:?}", err);
            err
        })?;
    debug!("metrics.zip downloaded to {}", "./buffer/metrics.zip");

    // download submissions.zip (the filings metadata)
    debug!("downloading submissions.zip ...");
    download_file(&http_client, SUBMISSIONS_URL, "./buffer/metrics.zip")
        .await
        .map_err(|err| {
            error!("failed to download submissions.zip: {:?}", err);
            err
        })?;
    debug!(
        "submissions.zip downloaded to {}",
        "./buffer/submissions.zip"
    );

    // unzip the files, using an async stream

    Ok(())
}

fn build_client() -> HttpClient {
    reqwest::ClientBuilder::new()
        .user_agent(var("USER_AGENT").expect("failed to read USER_AGENT"))
        .build()
        .expect("failed to build reqwest client")
}
