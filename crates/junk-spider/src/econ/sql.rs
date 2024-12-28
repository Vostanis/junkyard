/// Insert a dataset row from a FRED data source.
pub(crate) static INSERT_METRIC: &'static str = "
    INSERT INTO econ.fred (dated, metric, val) VALUES ($1, $2, $3)
    ON CONFLICT (dated, metric, val) DO NOTHING
";
