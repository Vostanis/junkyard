mod sql;

/// [Binance API](https://developers.binance.com/docs/binance-spot-api-docs/rest-api/public-api-endpoints)
pub mod binance;

/// [Kraken API](https://docs.kraken.com/api/docs/rest-api/get-ohlc-data)
pub mod kraken;

/// [KuCoin API](https://www.kucoin.com/docs/rest/spot-trading/market-data/get-klines)
pub mod kucoin;

/// [MEXC API](https://mexcdevelop.github.io/apidocs/spot_v3_en/#kline-candlestick-data)
pub mod mexc;

/// Common utilities for crypto exchanges
mod util {
    use crate::http::PgClient;
    use anyhow::Result;
    use rayon::prelude::{IntoParallelIterator, ParallelIterator};
    use std::collections::HashMap as Map;
    use tokio_postgres::types::FromSql;

    /// Retrieve a Map of <Symbol Names: Primary Keys>
    pub(crate) async fn fetch_pks<K, V>(
        pg_client: &mut PgClient,
        query: &str,
        key_col: &str,
        val_col: &str,
    ) -> Result<Map<K, V>>
    where
        K: Send + Sync + std::hash::Hash + Eq + for<'a> FromSql<'a>,
        V: Send + Sync + for<'a> FromSql<'a>,
    {
        let map: Map<K, V> = pg_client
            .query(query, &[])
            .await?
            .into_par_iter()
            .map(|row| {
                let key: K = row.get(key_col);
                let val: V = row.get(val_col);
                (key, val)
            })
            .collect();
        Ok(map)
    }
}
