/// Spider API calls are made up the following components:
/// 1. HTTP
///     a) client
///     b) request
///     c) deserializer
///     d) OPTIONAL: transformation
///
/// 2. PostgreSQL
///     a) connection
///     b) query
///     c) insert/copy process
///     d) OPTIONAL: key tracking
trait API {}
