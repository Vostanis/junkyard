#![allow(dead_code)]

use std::collections::HashMap;

// interval pk mappings
lazy_static::lazy_static! {
    pub(crate) static ref INTERVAL_PKS: HashMap<String, i32> = {
        let mut map = HashMap::new();
        map.insert("30m".to_string(), 1);
        map.insert("1h".to_string(), 2);
        map.insert("1d".to_string(), 3);
        map.insert("1w".to_string(), 4);
        map
    };
}

///////////////////////////////////////////////////////
// prices
///////////////////////////////////////////////////////

// insert price cell
pub(crate) const INSERT_PRICE: &'static str = "
    INSERT INTO crypto.prices (
        symbol_pk, 
        time, 
        interval_pk, 
        opening, 
        high, 
        low, 
        closing, 
        volume, 
        trades, 
        source_pk
    )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
    ON CONFLICT (symbol_pk, time, interval_pk, source_pk)
    DO NOTHING
";

///////////////////////////////////////////////////////
// sources
///////////////////////////////////////////////////////

// insert source
pub(crate) const INSERT_SOURCE: &'static str = "
    INSERT INTO crypto.sources (pk, source)
    VALUES ($1)
    ON CONFLICT (source)
    DO NOTHING
";

// return source primary key
pub(crate) const SELECT_SOURCE_PK: &'static str = "
    SELECT pk FROM crypto.sources
    WHERE sources = $1
";

///////////////////////////////////////////////////////
// symbols
///////////////////////////////////////////////////////

// insert symbol
pub(crate) const INSERT_SYMBOL: &'static str = "
    INSERT INTO crypto.symbols (symbol)
    VALUES ($1)
    ON CONFLICT (symbol)
    DO NOTHING
";

// return symbol primary key
pub(crate) const SELECT_SYMBOL_PK: &'static str = "
    SELECT pk FROM crypto.symbols
    WHERE symbol = $1
";
