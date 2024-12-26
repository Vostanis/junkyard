use crate::fs::{download_file, unzip};
use crate::http::*;
use tracing::{debug, error, info};

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
pub async fn scrape() -> anyhow::Result<()> {
    let http_client = build_client();

    info!("downloading companyfacts.zip and submissions.zip ...");

    // download companyfacts.zip (the metrics)
    debug!("downloading metrics.zip");
    download_file(&http_client, METRICS_URL, "./buffer/metrics.zip")
        .await
        .map_err(|err| {
            error!("failed to download metrics.zip: {:?}", err);
            err
        })?;
    debug!("metrics.zip downloaded to {}", "./buffer/metrics.zip");

    // download submissions.zip (the filings metadata)
    debug!("downloading submissions.zip");
    download_file(&http_client, SUBMISSIONS_URL, "./buffer/submissions.zip")
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
    info!("unzipping metrics.zip and submissions.zip ...");

    debug!("unzipping metrics.zip");
    unzip("./buffer/metrics.zip", "./buffer/metrics")
        .await
        .map_err(|err| {
            error!("failed to unzip metrics.zip: {:?}", err);
            err
        })?;
    debug!(
        "metrics.zip unzipped successfully to {}",
        "./buffer/metrics"
    );

    debug!("unzipping submissions.zip");
    unzip("./buffer/submissions.zip", "./buffer/submissions")
        .await
        .map_err(|err| {
            error!("failed to unzip submissions.zip: {:?}", err);
            err
        })?;
    debug!(
        "submissions.zip unzipped successfully to {}",
        "./buffer/submissions"
    );

    debug!("deleting metrics.zip and submissions.zip");
    tokio::fs::remove_dir("./buffer/metrics.zip").await?;
    tokio::fs::remove_dir("./buffer/submissions.zip").await?;

    Ok(())
}

fn build_client() -> HttpClient {
    reqwest::ClientBuilder::new()
        .user_agent(var("USER_AGENT").expect("failed to read USER_AGENT"))
        .build()
        .expect("failed to build reqwest client")
}
