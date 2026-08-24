use mcrl2::ATerm as Mcrl2ATerm;
use mcrl2::LinearProcessSpecification;
use mcrl2::mcrl2_aterm_to_merc;
use merc_data::BasicSort;
use merc_data::DataEquation;
use merc_data::DataFunctionSymbol;
use merc_data::Mcrl2DataSpecification;
use merc_data::SortAlias;

/// Converts `lps`'s data specification into the pure-Rust [`merc_data`] representation that
/// [`merc_lts::write_lts`] and [`merc_symbolic::write_symbolic_lts`] require, translating every
/// declaration's term via [`mcrl2::mcrl2_aterm_to_merc`] so the result no longer depends on the
/// mCRL2 C++ term pool.
pub fn convert_data_specification(lps: &LinearProcessSpecification) -> Mcrl2DataSpecification {
    let convert = |term: Mcrl2ATerm| mcrl2_aterm_to_merc(&term.copy());

    let ffi_data_spec = lps.data_specification();
    Mcrl2DataSpecification::new(
        ffi_data_spec
            .user_defined_sorts()
            .to_vec()
            .into_iter()
            .map(|t| BasicSort::from(convert(t)))
            .collect(),
        ffi_data_spec
            .user_defined_aliases()
            .to_vec()
            .into_iter()
            .map(|t| SortAlias::from(convert(t)))
            .collect(),
        ffi_data_spec
            .user_defined_constructors()
            .to_vec()
            .into_iter()
            .map(|t| DataFunctionSymbol::from(convert(t)))
            .collect(),
        ffi_data_spec
            .user_defined_mappings()
            .to_vec()
            .into_iter()
            .map(|t| DataFunctionSymbol::from(convert(t)))
            .collect(),
        ffi_data_spec
            .user_defined_equations()
            .to_vec()
            .into_iter()
            .map(|t| DataEquation::from(convert(t)))
            .collect(),
    )
}
