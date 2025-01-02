use crate::fs::{download_file, unzip};
use tracing::{debug, error};

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
pub async fn scrape(tui: bool) -> anyhow::Result<()> {
    let http_client = crate::std_client_build();

    // download companyfacts.zip (the metrics)
    debug!("downloading metrics.zip");
    if tui {
        println!(
            "{bar}\n{name:^40}\n{bar}",
            bar = "=".repeat(40),
            name = "metrics.zip"
        );
    }
    download_file(&http_client, METRICS_URL, "./buffer/metrics.zip", tui)
        .await
        .map_err(|err| {
            error!("failed to download metrics.zip: {:?}", err);
            err
        })?;
    debug!("metrics.zip downloaded to {}", "./buffer/metrics.zip");

    debug!("unzipping metrics.zip");
    unzip("./buffer/metrics.zip", "./buffer/metrics", tui)
        .await
        .map_err(|err| {
            error!("failed to unzip metrics.zip: {:?}", err);
            err
        })?;
    debug!(
        "metrics.zip unzipped successfully to {}",
        "./buffer/metrics"
    );

    if tui {
        println!("metrics downloaded\n")
    }

    // download submissions.zip (the filings metadata)
    debug!("downloading submissions.zip");
    if tui {
        println!(
            "{bar}\n{name:^40}\n{bar}",
            bar = "=".repeat(40),
            name = "submissions.zip"
        )
    }
    download_file(
        &http_client,
        SUBMISSIONS_URL,
        "./buffer/submissions.zip",
        tui,
    )
    .await
    .map_err(|err| {
        error!("failed to download submissions.zip: {:?}", err);
        err
    })?;
    debug!(
        "submissions.zip downloaded to {}",
        "./buffer/submissions.zip"
    );

    debug!("unzipping submissions.zip");
    unzip("./buffer/submissions.zip", "./buffer/submissions", tui)
        .await
        .map_err(|err| {
            error!("failed to unzip submissions.zip: {:?}", err);
            err
        })?;
    debug!(
        "submissions.zip unzipped successfully to {}",
        "./buffer/submissions"
    );

    if tui {
        println!("submissions downloaded\n");
    }

    // clean up the zips
    debug!("deleting metrics.zip and submissions.zip");
    tokio::fs::remove_file("./buffer/metrics.zip").await?;
    tokio::fs::remove_file("./buffer/submissions.zip").await?;

    if tui {
        println!("metrics.zip and submissions.zip deleted\n");
    }

    Ok(())
}
