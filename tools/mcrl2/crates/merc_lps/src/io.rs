
#[derive(Clone, Debug)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum LpsFormat {
    /// The standard mCRL2 LPS format.
    Lps,

    /// A human readable text format using the mCRL2 syntax.
    Text,
}