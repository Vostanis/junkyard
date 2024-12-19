///////////////////////////////////////////////////////
// prices
///////////////////////////////////////////////////////

// insert price cell
pub(crate) const INSERT_PRICE: &'static str = "
    INSERT INTO crypto.prices (
        pk, 
        time, 
        interval, 
        opening, 
        high, 
        low, 
        closing, 
        volume, 
        trades, 
        quote_asset_volume, 
        exchange
    )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
    ON CONFLICT (pk, time, interval, exchange)
    DO NOTHING
";

///////////////////////////////////////////////////////
// sources
///////////////////////////////////////////////////////

// insert source
pub(crate) const INSERT_SOURCE: &'static str = "
    INSERT INTO crypto.sources (pk, source)
    VALUES ($1)
    ON CONFLICT (pk)
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
    ON CONFLICT (pk)
    DO NOTHING
";

// return symbol primary key
pub(crate) const SELECT_SYMBOL_PK: &'static str = "
    SELECT pk FROM crypto.symbols
    WHERE symbol = $1
";
