pub mod crypto;
// pub mod econ;
pub mod stock;

/// Shortcut for required API elements.
pub(crate) mod http {
    pub(crate) use dotenv::var;
    pub(crate) use reqwest::Client as HttpClient;
    pub(crate) use tokio_postgres::Client as PgClient;
}
