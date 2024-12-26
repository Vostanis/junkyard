use crate::http::*;
use std::sync::Arc;
use tokio::fs::File;
use tokio::sync::Mutex;
use tracing::trace;

// 1. check if downloads are necessary
// 2. download if necessary
//     a. metrics
//     b. submissions

const METRICS_URL: &'static str =
    "https://www.sec.gov/Archives/edgar/daily-index/xbrl/companyfacts.zip";

const SUBMISSIONS_URL: &'static str =
    "https://www.sec.gov/Archives/edgar/daily-index/bulkdata/submissions.zip";

const CHUNK_SIZE: u64 = 100 * 1024 * 1024; // 100 MB

async fn scrape() -> anyhow::Result<()> {
    let response = client
        .get("https://www.sec.gov/files/company_tickers.json")
        .send()
        .await?;
    let body = response.json().await?;
    Ok(())
}

/// GET request a file from `url` and write it to `path`, parallelising
/// the download process with [`rayon`].
///
/// [`rayon`]: https://docs.rs/rayon/latest/rayon/
async fn download_file(http_client: &HttpClient, url: &str, path: &str) -> anyhow::Result<()> {
    use reqwest::header::CONTENT_LENGTH;

    let client = http_client;

    // Get the content length from the URL header
    let response = client.get(url).send().await?;
    let file_size = response
        .headers()
        .get(CONTENT_LENGTH)
        .and_then(|len| len.to_str().ok())
        .and_then(|len| len.parse::<u64>().ok())
        .unwrap_or(0);

    // Ensure the directory exists
    let dir_path = std::path::Path::new(path)
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Failed to get directory path"))?;
    tokio::fs::create_dir_all(dir_path).await?;

    // Initialise central variables of async process
    let file = File::create(path).await?;
    let file = Arc::new(Mutex::new(file));
    let num_chunks = (file_size + CHUNK_SIZE - 1) / CHUNK_SIZE;
    let mut tasks = Vec::with_capacity(num_chunks as usize);

    // Build each async task and push to tasks
    for i in 0..num_chunks {
        let start = i * CHUNK_SIZE;
        let end = std::cmp::min((i + 1) * CHUNK_SIZE, file_size);
        let client = &client;
        let url = url.to_string();
        let file = file.clone();
        tasks.push(tokio::spawn(async move {
            let mut file = file.lock().await;
            let _download_chunk = client.download_chunk(&url, start, end, &mut file).await;
        }));
    }

    // Join all async tasks together, in order to execute
    let mut outputs = Vec::with_capacity(tasks.len());
    for task in tasks {
        outputs.push(task.await.unwrap());
        // sleep(std::time::Duration::from_secs(1)).await;
    }

    // Finish the progress bar
    let file = Arc::try_unwrap(file).unwrap().into_inner();
    trace!(
        "{} downloaded",
        indicatif::HumanBytes(file.metadata().await?.len())
    );

    Ok(())
}

/// Download a range of bytes (a chunk) with a GET request.
async fn download_chunk(
    http_client: &HttpClient,
    url: &str,
    start: u64,
    end: u64,
    output_file: &mut File,
) -> anyhow::Result<()> {
    let client = http_client;
    let url = url.to_string();
    let range = format!("bytes={}-{}", start, end - 1);

    // download a range of bytes
    let response = client
        .get(url)
        .header(reqwest::header::RANGE, range)
        .send()
        .await?;

    // seek the position of bytes and write to the file
    let body = response.bytes().await?;
    let _seek = output_file.seek(tokio::io::SeekFrom::Start(start)).await?;
    let _write = output_file.write_all(&body).await?;
    trace!(
        "downloaded chunk: {}",
        indicatif::HumanBytes(body.len().try_into().expect("usize to u64"))
    );
    Ok(())
}
