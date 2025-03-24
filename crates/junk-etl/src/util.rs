pub(crate) fn num_concurrent_threads() -> usize {
    dotenv::var("CONCURRENT_THREADS")
        .expect("CONCURRENT_THREADS not found")
        .parse::<usize>()
        .expect("invalid CONCURRENT_THREADS format - must a valid integer")
}

