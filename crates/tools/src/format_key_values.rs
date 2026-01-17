use std::io;
use std::io::Write;

use env_logger::fmt::Formatter;
use log::kv::Key;
use log::kv::Source;
use log::kv::Value;
use log::kv::VisitSource;

/// Formats key-value pairs from the given source as a JSON object.
///
/// # Details
///
/// Can be used to configure `env_logger` to format log key-values as JSON.
///
/// ```rust
///
/// env_logger::Builder::new()
///     .format_key_values(|formatter, source| {
///         format_key_values_json(formatter, source)?;
///     })
///     .init();
///
/// ```
pub fn format_key_values_json(formatter: &mut Formatter, source: &dyn Source) -> io::Result<()> {
    if source.count() > 0 {
        // If there are key-values, format them as a JSON object.
        formatter.write_all("\n{ ".as_bytes())?;
        let mut json_printer = JsonPrinter { formatter };
        if let Err(e) = source.visit(&mut json_printer) {
            return Err(io::Error::new(io::ErrorKind::InvalidData, e));
        }
        formatter.write_all(" }".as_bytes())?;
    }

    Ok(())
}

/// A visitor that formats key-value pairs as JSON on a single line.
struct JsonPrinter<'a> {
    formatter: &'a mut Formatter,
}

impl<'a, 'kvs> VisitSource<'kvs> for JsonPrinter<'a> {
    fn visit_pair(&mut self, key: Key<'kvs>, value: Value<'kvs>) -> Result<(), log::kv::Error> {
        self.formatter.write_fmt(format_args!(r#""{}": {}"#, key, value))?;
        Ok(())
    }
}
