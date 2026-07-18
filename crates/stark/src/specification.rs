//! The checked form of a STARK specification.
//!
//! Parsing yields an [UntypedStarkSpecification]: a faithful syntax tree whose
//! references are unresolved (`DefRef::id` is `None`) and whose expressions have
//! no types yet. Running [UntypedStarkSpecification::check] — name resolution
//! followed by type checking, both from `RESOLVE_TYPECHECK_PLAN.md` — either
//! reports every problem at once through [Diagnostics] or produces a
//! [StarkSpecification], which pairs the now fully-resolved tree with the
//! [SymbolTable] and [TypeTable] that describe it.
//!
//! The type distinction is the point: only `check` can produce a
//! [StarkSpecification], so anything holding one (a future model lowering or
//! evaluator) knows resolution and type checking already succeeded and never has
//! to re-derive or re-validate that.

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
}

impl UntypedStarkSpecification {
    /// Resolves and type-checks this specification.
    ///
    /// Type checking runs even when resolution reported errors — unresolved
    /// references simply stay `None` and the checker skips them — so a single
    /// call reports the problems from both passes together rather than making
    /// the caller fix all the name errors before seeing any type errors.
    pub fn check(mut self) -> Result<StarkSpecification, Diagnostics> {
        let (symbols, mut diagnostics) = resolve(&mut self);
        let (types, type_diagnostics) = typecheck(&self, &symbols);
        diagnostics.extend(type_diagnostics);

        diagnostics.into_result(StarkSpecification {
            ast: self,
            symbols,
            types,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::UntypedStarkSpecification;

    #[test]
    fn checks_every_example_specification() {
        for (name, source) in [
            ("engine.stark", include_str!("../../../examples/stark/engine.stark")),
            ("random_walk.stark", include_str!("../../../examples/stark/random_walk.stark")),
            (
                "single_vehicle.stark",
                include_str!("../../../examples/stark/single_vehicle.stark"),
            ),
            ("toll.stark", include_str!("../../../examples/stark/toll.stark")),
            (
                "two_vehicles.stark",
                include_str!("../../../examples/stark/two_vehicles.stark"),
            ),
            ("monitoring.stark", include_str!("../../../examples/stark/monitoring.stark")),
            ("agriculturalDT.stark", include_str!("../../../examples/stark/agriculturalDT.stark")),
        ] {
            let spec = UntypedStarkSpecification::parse(source).unwrap_or_else(|e| panic!("{name} failed to parse: {e}"));

            if let Err(diagnostics) = spec.check() {
                panic!("{name} failed to check:\n{}", diagnostics.render(source));
            }
        }
    }

    #[test]
    fn reports_resolve_and_type_errors_together() {
        let source = "const c = missing_name; const d = 1 + true;";
        let spec = UntypedStarkSpecification::parse(source).expect("should parse");

        let diagnostics = spec.check().err().expect("should not check");
        assert!(
            diagnostics.items().len() >= 2,
            "expected both passes to report: {diagnostics}"
        );
    }
}
