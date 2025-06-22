use std::fmt;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const BUILD_HASH: &str = env!("BUILD_HASH");

pub struct Version;

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", VERSION, &BUILD_HASH[..8])
    }
}
