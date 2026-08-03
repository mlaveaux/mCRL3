use crate::ast::UntypedStarkSpecification;
use crate::diagnostics::Diagnostics;
use crate::resolve::SymbolTable;
use crate::resolve::resolve;
use crate::typecheck::TypeTable;
use crate::typecheck::typecheck;

/// A STARK specification that has been resolved and type-checked.
pub struct StarkSpecification {
    ast: UntypedStarkSpecification,
    symbols: SymbolTable,
    types: TypeTable,
}

impl StarkSpecification {
    /// The underlying syntax tree, with every reference resolved.
    pub fn ast(&self) -> &UntypedStarkSpecification {
        &self.ast
    }

    /// What every `DefId`, `StateId` and `LocalId` in [Self::ast] refers to.
    pub fn symbols(&self) -> &SymbolTable {
        &self.symbols
    }

    /// The inferred type of every declaration and function signature.
    pub fn types(&self) -> &TypeTable {
        &self.types
    }

    /// Resolves and type-checks this untyped specification.
    ///
    /// Type checking runs even when resolution reported errors. It simply
    /// ignores unresolved entries and updates the presented Diagnostics.
    pub fn from_untyped(mut spec: UntypedStarkSpecification) -> Result<StarkSpecification, Diagnostics> {
        let (symbols, mut diagnostics) = resolve(&mut spec);
        let (types, type_diagnostics) = typecheck(&spec, &symbols);
        diagnostics.extend(type_diagnostics);

        if diagnostics.has_errors() {
            log::debug!(
                "specification rejected with {} diagnostic(s)",
                diagnostics.items().len()
            );
        } else {
            log::debug!("specification checked successfully");
        }

        diagnostics.into_result(StarkSpecification {
            ast: spec,
            symbols,
            types,
        })
    }
}

#[cfg(test)]
mod tests {
    use test_log::test;

    use crate::ast::UntypedStarkSpecification;
    use crate::specification::StarkSpecification;

    #[test]
    fn reports_resolve_and_type_errors_together() {
        let source = "const c = missing_name; const d = 1 + true;";
        let spec = UntypedStarkSpecification::parse(source).expect("should parse");

        let diagnostics = StarkSpecification::from_untyped(spec).err().expect("should not check");
        assert!(
            diagnostics.items().len() >= 2,
            "expected both passes to report: {diagnostics}"
        );
    }
}
