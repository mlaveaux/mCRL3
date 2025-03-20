use std::fmt;
use std::ops::Deref;

/// Returns a reference to the term pool
///
/// TODO: Keep a guard to disallow resizing while the reference is alive.
#[derive(Debug)]
pub struct StrRef<'a> {
    name: &'a str,
}

impl fmt::Display for StrRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl<'a> StrRef<'a> {
    pub fn new(name: &str) -> StrRef {
        StrRef { name }
    }
}

impl Deref for StrRef<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.name
    }
}
