


/// Variant of variability solver to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum Solver {
    /// Zielonka's recursive algorithm.
    Zielonka,
    /// Priority promotion algorithm.
    PriorityPromotion,
}

/// Variant of the parity game algorithm to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum ZielonkaVariant {
    /// Product-based Zielonka variant.
    Product,
    /// Standard family-based Zielonka algorithm.
    Family,
    /// Left-optimised family-based Zielonka variant.
    FamilyOptimisedLeft,
}