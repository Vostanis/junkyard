use crate::http::*;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio::sync::Mutex;
use tracing::{debug, error, info, trace};

const CHUNK_SIZE: u64 = 100 * 1024 * 1024; // 100 MB

/// GET request a file from `url` and write it to `path`, parallelising
/// the download process with [`rayon`].
///
/// [`rayon`]: https://docs.rs/rayon/latest/rayon/
pub async fn download_file(http_client: &HttpClient, url: &str, path: &str) -> anyhow::Result<()> {
    use reqwest::header::CONTENT_LENGTH;

    let client = http_client.clone();

    // get the content length from the URL header
    let response = client.get(url).send().await?;
    let file_size = response
        .headers()
        .get(CONTENT_LENGTH)
        .and_then(|len| len.to_str().ok())
        .and_then(|len| len.parse::<u64>().ok())
        .unwrap_or(0);

    // ensure the directory exists
    trace!("checking directory path: {:?}", path);
    let dir_path = std::path::Path::new(path)
        .parent()
        .ok_or_else(|| anyhow::anyhow!("failed to get directory path"))?;
    tokio::fs::create_dir_all(dir_path).await?;

    // initialise central variables of async process
    let file = File::create(path).await?;
    let file = Arc::new(Mutex::new(file));
    let num_chunks = (file_size + CHUNK_SIZE - 1) / CHUNK_SIZE;
    let mut tasks = Vec::with_capacity(num_chunks as usize);

    // build each async task and push to variable `tasks`; each task downloading a chunk of data
    for i in 0..num_chunks {
        let start = i * CHUNK_SIZE;
        let end = std::cmp::min((i + 1) * CHUNK_SIZE, file_size);
        let url = url.to_string();
        let file = file.clone();
        let client = client.clone();
        tasks.push(tokio::spawn(async move {
            let mut file = file.lock().await;
            let _chunk = download_chunk(&client, &url, start, end, &mut file).await;
        }));
    }

    // join all async tasks together, in order to execute
    let mut outputs = Vec::with_capacity(tasks.len());
    for task in tasks {
        outputs.push(task.await.unwrap());
    }

    Ok(())
}

/// Download a range of bytes (a chunk) with a GET request.
pub async fn download_chunk(
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

    Ok(())
}

/// Reads a `.json` file from `path`.
///
/// ```rust
/// let ouput: DesiredType = renai_client::read_json(path).await?;
/// ```
pub async fn read_json<T: serde::de::DeserializeOwned>(path: &str) -> anyhow::Result<T> {
    let file = tokio::fs::read(path).await?;
    let data: T = serde_json::from_slice(&file)?;
    Ok(data)
}

/// Unzip a `.zip` file (`zip_file`) to a target directory (`to_dir`).
///
/// `std::fs::create_dir_all(to_dir)?` is used in creating `to_dir` path,
/// so directories will be created, as necessary, by the unzip() function.
pub async fn unzip(zip_file: &str, to_dir: &str) -> anyhow::Result<()> {
    debug!("unzipping {zip_file} to {to_dir}");

    // use of rayon requires lots of async wrappings
    let file = std::fs::File::open(zip_file)?;
    let archive = zip::ZipArchive::new(file).map_err(|err| {
        error!("failed to open zip file at {}, {}", zip_file, err);
        err
    })?;
    let zip_length = archive.len();
    let archive = Arc::new(std::sync::Mutex::new(archive));

    // ensure the target directory exists
    tokio::fs::create_dir_all(to_dir).await?;

    // parallel iteration across zipped files
    (0..zip_length).into_par_iter().for_each(|i| {
        let archive = archive.clone();
        let mut archive = archive.lock().expect("unlock zip archive");
        let mut file = archive.by_index(i).expect("file from zip archive");
        let outpath = format!("{to_dir}/{}", file.mangled_name().display());
        let outdir = std::path::Path::new(&outpath)
            .parent()
            .expect("parent directory of output path");
        if !outdir.exists() {
            std::fs::create_dir_all(&outdir).expect("failed to create directory");
        }

        // fxtract the file
        let mut outfile = std::fs::File::create(&outpath).expect("creation of output file");
        trace!("copying {} to {}", file.name(), outpath);
        std::io::copy(&mut file, &mut outfile).expect("copying of zip file to output");
    });

    info!("{zip_file} unzipped to {to_dir}");

    Ok(())
}
