// use anyhow::Result;
// use deadpool_postgres::Pool;
// use futures::{stream, StreamExt};
// use reqwest::header::{HeaderMap, HeaderValue};
// use reqwest::{Client as HttpClient, ClientBuilder};
// use serde::Deserialize;
// use tokio_postgres::Client as PgClient;
// use tracing::{debug, error, info};

// /// The HTTP webscraping subprocess (fetch the data from the web).
// pub(super) trait HttpGet {
//     async fn http_get(http_client: HttpClient) -> Result<Self>
//     where
//         Self: for<'de> Deserialize<'de>;
// }

// /// The PostgreSQL Loading subprocess (place the data into the database).
// pub(super) trait PgLoad {
//     async fn pg_load(&self, pg_client: &mut PgClient) -> Result<()>;
// }

// /// The entirety of the Crypto-broker-scraping framework.
// ///
// /// `execute()` is designed to be the only function called outside of the crate.
// pub(super) trait Broker<Symbols, Prices>
// where
//     Symbols: for<'de> Deserialize<'de> + std::iter::Iterator + HttpGet + PgLoad,
//     Prices: for<'de> Deserialize<'de> + std::iter::Iterator + HttpGet + PgLoad,
// {
//     /// Convenient way of retrieving Type Name for debugging, e.g.
//     ///
//     /// ```rust
//     /// impl Broker for Binance {
//     ///     // `type_name()` would return `Binance`.
//     /// }
//     /// ```
//     fn name() -> &'static str;

//     /// Simplified HTTP ClientBuilder; just define a vector of the
//     /// static header names, and the resulting way to retrieve their
//     /// values.
//     ///
//     /// This saves the user from redefining the actual ClientBuilder
//     /// every time.
//     fn http_headers<'a>() -> Vec<(&'static str, String)>;

//     /// Build a HTTP Client for making the requests.
//     fn http_client() -> Result<HttpClient> {
//         let headers = Self::http_headers();

//         // Loop the given headers, and insert them to a map.
//         let mut header_map = HeaderMap::new();
//         for (label, variable) in headers.iter() {
//             let api_header = HeaderValue::from_str(variable).map_err(|e| {
//                 error!(
//                     "Failed to insert {variable:?} into Client Headers for {}: {e}",
//                     Self::name()
//                 );
//                 e
//             })?;
//             header_map.insert(*label, api_header);
//         }

//         // Build a client from the Headermap.
//         let client = ClientBuilder::new()
//             .default_headers(header_map)
//             .build()
//             .map_err(|e| {
//                 error!(
//                     "Failed to build Client from HeaderMap for {}: {e}",
//                     Self::name()
//                 );
//                 e
//             })?;

//         Ok(client)
//     }

//     /// The process for returning the list of symbols for which the Broker webscraping is centered
//     /// around.
//     ///
//     /// The list return data type requires an implementation of [serde::Deserialize].
//     async fn symbols(&self, pool: &Pool) -> Result<Symbols>;

//     /// Use the symbols to GET & Load the prices.
//     async fn prices(&self, pool: &Pool) -> Result<()>;

//     /// Each Broker can use slightly different interval terminology, e.g.
//     ///
//     /// ```rust
//     ///     [ "1h", "1d", "1w" ], // default implementation
//     ///     [ "1hr", "1day", "1wk" ],
//     /// ```
//     fn intervals() -> Vec<&'static str> {
//         vec!["1h", "1d", "1w"]
//     }

//     /// Default framework for the entire webscraping process.
//     async fn execute(pool: &Pool) -> Result<()> {
//         // Fetch the initial Symbols list, and Load them to the database (without
//         // deaallocating them).
//         info!("Getting symbols for {} ...", Self::name());
//         let symbols = Self::symbols(pool).await?;

//         // We only need the 1 Source PK, so retrieve it; and if it fails then
//         // we insert a new Primary Key and call it again.
//         //
//         // In the event of another failure, the process ends.
//         info!(
//             "Retrieving existing Primary Keys for source: {} ...",
//             Self::name()
//         );
//         let source_pk = {
//             let source = Self::name();
//             super::common::existing_source(pool, source.to_string()).await?
//         };

//         // Fetch all the existing Symbol Primary Keys from the database.
//         info!("Retrieving existing Primary Keys for symbols ...");
//         let symbol_pk_map = super::common::existing_symbols(pool).await?;

//         // Using the Source PK, and having access to the existing_symbols map for `symbol_pk`s,
//         // fetch each price dataset.
//         info!("Fetching prices for {} ...", Self::name());
//         let http_client = Self::http_client();
//         let (mut success, mut failure): (u16, u16) = (0, 0);
//         let mut stream = stream::iter(symbols);
//         while let Some(symbol) = stream.next().await {
//             let http_client = &http_client;

//             async move {}.await;
//         }

//         // Finish.
//         info!("Collected prices from {}.", Self::name());

//         Ok(())
//     }
// }
