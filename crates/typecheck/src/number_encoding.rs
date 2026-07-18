/// Selects how the numeric sorts `Pos`, `Nat`, `Int` and `Real` are represented.
///
/// The choice affects two things that must always agree, which is why they are
/// driven by this single option:
///
/// 1. **The system specification** pulled in for the basic and container sorts —
///    either the recursive `pos.mcrl2` / `nat.mcrl2` / … templates or their
///    machine-word `pos64.mcrl2` / `nat64.mcrl2` / … counterparts (the latter
///    additionally pulling in `machine_word.mcrl2`).
/// 2. **How numeric literals are lowered**, since each specification defines a
///    different set of constructors for `Pos` and `Nat`.
///
/// Mixing the two would produce terms that no equation matches, so a
/// [`crate::DataSpecification`] records the encoding it was built with and
/// lowers literals accordingly.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum NumberEncoding {
    /// The recursive binary encoding of mCRL2's Appendix B: a `Pos` is the
    /// bit chain `@c1` / `@cDub(bit, p)` (denoting `2*p + bit`) and a `Nat` is
    /// `@c0` or `@cNat(p)`.
    ///
    /// This is the default, and the encoding merc has always used.
    #[default]
    Binary,

    /// The 64-bit machine-word encoding: a `Pos` is a base-`2^64` digit chain
    /// `@most_significant_digit(w)` / `@concat_digit(p, w)` (denoting
    /// `2^64 * p + w`) and a `Nat` is the analogous
    /// `@most_significant_digitNat(w)` / `@concat_digit(n, w)`, where each digit
    /// `w` is a `@word` machine number.
    ///
    /// Arithmetic on the digits is performed by the native `@word` operations
    /// (see `merc_number::machine_word`), so this encoding is substantially
    /// faster on large numbers than the recursive one.
    MachineWord,
}

impl NumberEncoding {
    /// Whether this encoding represents numbers as machine-word digits.
    pub fn is_machine_word(self) -> bool {
        matches!(self, NumberEncoding::MachineWord)
    }
}
