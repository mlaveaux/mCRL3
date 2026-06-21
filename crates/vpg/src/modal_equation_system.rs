use std::collections::HashSet;
use std::fmt;
use std::ops::ControlFlow;

use log::debug;

use merc_syntax::FixedPointOperator;
use merc_syntax::StateFrm;
use merc_syntax::StateVarDecl;
use merc_syntax::apply_statefrm;
use merc_syntax::visit_statefrm;

/// A fixpoint equation system representing a ranked set of fixpoint equations.
///
/// Each equation is of the shape `{mu, nu} X(args...) = rhs`. Where rhs
/// contains no further fixpoint equations.
pub struct ModalEquationSystem {
    equations: Vec<Equation>,
}

/// A single fixpoint equation of the shape `{mu, nu} X(args...) = rhs`.
#[derive(Clone)]
pub struct Equation {
    operator: FixedPointOperator,
    variable: StateVarDecl,
    rhs: StateFrm,
}

impl Equation {
    /// Returns the operator of the equation.
    pub fn operator(&self) -> FixedPointOperator {
        self.operator
    }

    /// Returns the variable declaration of the equation.
    pub fn variable(&self) -> &StateVarDecl {
        &self.variable
    }

    /// Returns the body of the equation.
    pub fn body(&self) -> &StateFrm {
        &self.rhs
    }
}

impl From<Equation> for StateFrm {
    fn from(val: Equation) -> Self {
        StateFrm::FixedPoint {
            operator: val.operator,
            variable: val.variable,
            body: Box::new(val.rhs),
        }
    }
}

impl ModalEquationSystem {
    /// Converts a plain state formula into a fixpoint equation system.
    pub fn new(formula: &StateFrm) -> Self {
        let mut equations = Vec::new();
        let mut identifier_generator = FreshStateVarGenerator::new(formula);

        // Ensure that the formula has an outermost fixpoint operator.
        let formula = add_placeholder_operator(formula.clone(), &mut identifier_generator);

        // Apply E to extract all equations from the formula
        apply_e(&mut equations, &formula);

        // Check that there are no duplicate variable names
        let identifiers: HashSet<&String> = HashSet::from_iter(equations.iter().map(|eq| &eq.variable.identifier));
        assert_eq!(
            identifiers.len(),
            equations.len(),
            "Duplicate variable names found in fixpoint equation system"
        );

        debug_assert!(
            !equations.is_empty(),
            "At least one fixpoint equation expected in the equation system"
        );

        ModalEquationSystem { equations }
    }

    /// Returns the ith equation in the system.
    pub fn equation(&self, i: usize) -> &Equation {
        &self.equations[i]
    }

    /// Returns the number of equations in the system.
    pub fn len(&self) -> usize {
        self.equations.len()
    }

    /// Returns true if the system contains no equations.
    pub fn is_empty(&self) -> bool {
        self.equations.is_empty()
    }

    /// The alternation depth is a complexity measure of the given formula.
    ///
    /// # Details
    ///
    /// The alternation depth of mu X . psi is defined as the maximum chain X <= X_1 <= ... <= X_n,
    /// where X <= Y iff X appears freely in the corresponding equation sigma Y . phi. And furthermore,
    /// X_0, X_2, ... are bound by mu and X_1, X_3, ... are bound by nu. Similarly, for nu X . psi. Note
    /// that the alternation depth of a formula with a rhs is always 1, since the chain cannot be extended.
    pub fn alternation_depth(&self, i: usize) -> usize {
        let equation = &self.equations[i];
        self.alternation_depth_rec(i, equation.body(), &equation.variable().identifier)
    }

    /// Finds an equation by its variable identifier.
    ///
    /// # Details
    ///
    /// This is a linear scan over the equations, and is also called from within
    /// [`Self::alternation_depth`]'s recursion. Equation systems correspond to a
    /// single (modal) formula and are therefore small, so an index map is not
    /// worth its maintenance cost; revisit if very large formulas become common.
    pub fn find_equation_by_identifier(&self, id: &str) -> Option<(usize, &Equation)> {
        self.equations
            .iter()
            .enumerate()
            .find(|(_, eq)| eq.variable.identifier == id)
    }

    /// Recursive helper function to compute the alternation depth of equation `i`.
    fn alternation_depth_rec(&self, i: usize, formula: &StateFrm, identifier: &String) -> usize {
        let equation = &self.equations[i];

        match formula {
            StateFrm::Id(id, _) => {
                if id == identifier {
                    1
                } else {
                    let (j, inner_equation) = self
                        .find_equation_by_identifier(id)
                        .expect("Equation not found for identifier");
                    if j > i {
                        let depth = self.alternation_depth_rec(j, &inner_equation.rhs, identifier);
                        depth
                            + (if inner_equation.operator != equation.operator {
                                1 // Alternation occurs.
                            } else {
                                0
                            })
                    } else {
                        // Only consider nested equations
                        0
                    }
                }
            }
            StateFrm::Binary { lhs, rhs, .. } => self
                .alternation_depth_rec(i, lhs, identifier)
                .max(self.alternation_depth_rec(i, rhs, identifier)),
            StateFrm::Modality { expr, .. } => self.alternation_depth_rec(i, expr, identifier),
            StateFrm::True | StateFrm::False => 0,
            _ => {
                unimplemented!("Cannot determine alternation depth of formula {}", formula)
            }
        }
    }
}

/// If the given formula has no outermost fixpoint operator, adds a placeholder
/// fixpoint operator around it.
fn add_placeholder_operator(formula: StateFrm, identifier_generator: &mut FreshStateVarGenerator) -> StateFrm {
    if matches!(formula, StateFrm::FixedPoint { .. }) {
        // The outer operator is already a fixpoint
        formula
    } else {
        // Introduce a placeholder.
        StateFrm::FixedPoint {
            operator: FixedPointOperator::Least,
            variable: StateVarDecl::new(identifier_generator.generate("X"), Vec::new()),
            body: Box::new(formula),
        }
    }
}

/// Applies `E` to the given formula, adding equations to the given vector.
///
/// E(nu X. f) = (nu X = RHS(f)) + E(f)
/// E(mu X. f) = (mu X = RHS(f)) + E(f)
/// E(g) = ... (traverse all the subformulas of g and apply E to them)
fn apply_e(equations: &mut Vec<Equation>, formula: &StateFrm) {
    debug!("Applying E to formula: {}", formula);

    visit_statefrm::<(), _>(formula, |formula| match formula {
        StateFrm::FixedPoint {
            operator,
            variable,
            body,
        } => {
            debug!("Adding equation for variable {}", variable.identifier);
            // Add the equation with the renamed variable (the span is the same as the original variable).
            equations.push(Equation {
                operator: *operator,
                variable: variable.clone(),
                rhs: rhs(body),
            });

            Ok(ControlFlow::Continue(()))
        }
        _ => Ok(ControlFlow::Continue(())),
    })
    .expect("No error expected during fixpoint equation system construction");
}

/// Applies `RHS` to the given formula.
///
/// RHS(true) = true
/// RHS(false) = false
/// RHS(<a>f) = <a>RHS(f)
/// RHS([a]f) = [a]RHS(f)
/// RHS(f1 && f2) = RHS(f1) && RHS(f2)
/// RHS(f1 || f2) = RHS(f1) || RHS(f2)
/// RHS(X) = X
/// RHS(mu X. f) = X(args)
/// RHS(nu X. f) = X(args)
fn rhs(formula: &StateFrm) -> StateFrm {
    apply_statefrm(formula.clone(), |formula| match formula {
        // RHS(mu X. phi) = X(args)
        StateFrm::FixedPoint { variable, .. } => Ok(Some(StateFrm::Id(
            variable.identifier.clone(),
            variable.arguments.iter().map(|arg| arg.expr.clone()).collect(),
        ))),
        _ => Ok(None),
    })
    .expect("No error expected during RHS extraction")
}

/// A generator for fresh state variable names.
pub struct FreshStateVarGenerator {
    used: HashSet<String>,
}

impl FreshStateVarGenerator {
    /// Creates a new fresh state variable generator.
    ///
    /// # Details
    ///
    /// Traverses the given formula to collect all used variable names.
    pub fn new(formula: &StateFrm) -> Self {
        let mut used = HashSet::new();
        visit_statefrm::<(), _>(formula, |subformula| {
            if let StateFrm::FixedPoint { variable, .. } = subformula {
                used.insert(variable.identifier.clone());
            }

            Ok(ControlFlow::Continue(()))
        })
        .expect("No error expected during visiting");

        FreshStateVarGenerator { used }
    }

    /// Generates a fresh state variable name based on the given base.
    pub fn generate(&mut self, base: &str) -> String {
        let mut index = 0;
        loop {
            let candidate = format!("{}{}", base, index);
            if !self.used.contains(&candidate) {
                self.used.insert(candidate.clone());
                return candidate;
            }
            index += 1;
        }
    }
}

impl fmt::Display for ModalEquationSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, equation) in self.equations.iter().enumerate() {
            write!(f, "{i}: {} {} = {}", equation.operator, equation.variable, equation.rhs)?;
            if i + 1 < self.equations.len() {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use merc_macros::merc_test;
    use merc_syntax::UntypedStateFrmSpec;

    use super::ModalEquationSystem;

    #[merc_test]
    fn test_fixpoint_equation_system_construction() {
        let formula = UntypedStateFrmSpec::parse("mu X. [a]X && nu Y. <b>true")
            .unwrap()
            .formula;
        let fes = ModalEquationSystem::new(&formula);

        println!("{}", fes);

        assert_eq!(fes.equations.len(), 2);
        assert_eq!(fes.alternation_depth(0), 1);
        assert_eq!(fes.alternation_depth(1), 0);
    }

    #[merc_test]
    fn test_fixpoint_equation_system_example() {
        let formula = UntypedStateFrmSpec::parse(include_str!("../../../examples/vpg/running_example.mcf"))
            .unwrap()
            .formula;
        let fes = ModalEquationSystem::new(&formula);

        println!("{}", fes);

        assert_eq!(fes.equations.len(), 2);
        assert_eq!(fes.alternation_depth(0), 2);
        assert_eq!(fes.alternation_depth(1), 1);
    }

    #[merc_test]
    #[should_panic(expected = "Duplicate variable names found in fixpoint equation system")]
    fn test_fixpoint_equation_system_duplicates() {
        let formula = UntypedStateFrmSpec::parse("mu X. [a]X && (nu Y. <b>true) && (nu Y . <c>X)")
            .unwrap()
            .formula;
        let fes = ModalEquationSystem::new(&formula);

        println!("{}", fes);

        assert_eq!(fes.equations.len(), 3);
    }
}
