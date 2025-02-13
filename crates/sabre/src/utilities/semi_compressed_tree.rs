// Author(s): Mark Bouwman

use std::collections::HashMap;
use std::collections::HashSet;

use crate::utilities::ExplicitPosition;
use mcrl3_aterm::ATerm;
use mcrl3_aterm::ATermRef;
use mcrl3_aterm::Symbol;
use mcrl3_aterm::TermBuilder;
use mcrl3_aterm::TermPool;
use mcrl3_aterm::Yield;
use mcrl2::data::is_data_variable;
use mcrl2::data::DataVariable;

/// A SemiCompressedTermTree (SCTT) is a mix between a [ATerm] and a syntax tree and is used
/// to represent the rhs of rewrite rules and the lhs and rhs of conditions.
///
/// It stores as much as possible in the term pool. Due to variables it cannot be fully compressed.
/// For variables it stores the position in the lhs of a rewrite rule where the concrete term can be
/// found that will replace the variable.
///
/// # Examples
/// For the rewrite rule and(true, true) = true, the SCTT of the rhs will be of type Compressed, with
/// a pointer to the term true.
///
/// For the rewrite rule minus(x, 0) = x, the SCTT of the rhs will be of type Variable, storing position
/// 1, the position of x in the lhs.
///
/// For the rewrite rule minus(s(x), s(y)) = minus(x, y), the SCTT of the rhs will be of type
/// Explicit, which will stored the head symbol 'minus' and two child SCTTs of type Variable.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum SemiCompressedTermTree {
    Explicit(ExplicitNode),
    Compressed(ATerm),
    Variable(ExplicitPosition),
}

/// Stores the head symbol and a SCTT for every argument explicitly.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct ExplicitNode {
    pub head: Symbol,
    pub children: Vec<SemiCompressedTermTree>,
}

use SemiCompressedTermTree::*;

use super::PositionIndexed;
use super::PositionIterator;

pub type SCCTBuilder = TermBuilder<&'static SemiCompressedTermTree, &'static Symbol>;

impl SemiCompressedTermTree {
    /// Given an [ATerm] and a term pool this function instantiates the SCTT and computes a [ATerm].
    ///
    /// # Example
    /// For the SCTT belonging to the rewrite rule minus(s(x), s(y)) = minus(x, y)
    /// and the concrete lhs minus(s(0), s(0)) the evaluation will go as follows.
    /// evaluate will encounter an ExplicitNode and make two recursive calls to get the subterms.
    /// Both these recursive calls will return the term '0'.
    /// The term pool will be used to construct the term minus(0, 0).
    pub fn evaluate_with<'a>(&'a self, builder: &mut SCCTBuilder, t: &ATermRef<'_>, tp: &mut TermPool) -> ATerm {
        // TODO: Figure out if this can be done properly. This is safe because evaluate will always leave the
        // underlying vectors empty.
        let builder: &mut TermBuilder<&'a SemiCompressedTermTree, &'a Symbol> = unsafe { std::mem::transmute(builder) };

        builder
            .evaluate(
                tp,
                self,
                |_tp, args, node| {
                    match node {
                        Explicit(node) => {
                            // Create an ATerm with as arguments all the evaluated semi compressed term trees.
                            for i in 0..node.children.len() {
                                args.push(&node.children[i]);
                            }

                            Ok(Yield::Construct(&node.head))
                        }
                        Compressed(ct) => Ok(Yield::Term(ct.clone())),
                        Variable(p) => Ok(Yield::Term(t.get_position(p).protect())),
                    }
                },
                |tp, symbol, args| Ok(tp.create(symbol, args)),
            )
            .unwrap()
    }

    /// The same as [evaluate_with], but allocates a [SCCTBuilder] internally.
    pub fn evaluate(&self, t: &ATermRef<'_>, tp: &mut TermPool) -> ATerm {
        let mut builder = TermBuilder::<&SemiCompressedTermTree, &Symbol>::new();

        self.evaluate_with(&mut builder, t, tp)
    }

    /// Creates a SCTT from a term. The var_map parameter should specify where the variable can be
    /// found in the lhs of the rewrite rule.
    pub(crate) fn from_term(
        t: &ATermRef<'_>,
        var_map: &HashMap<DataVariable, ExplicitPosition>,
    ) -> SemiCompressedTermTree {
        if is_data_variable(t) {
            Variable(
                var_map
                    .get(&t.protect())
                    .unwrap_or_else(|| panic!("{t} not contained in variable mapping var_map"))
                    .clone(),
            )
        } else if t.arguments().is_empty() {
            Compressed(t.protect())
        } else {
            let children = t
                .arguments()
                .map(|c| SemiCompressedTermTree::from_term(&c, var_map))
                .collect();
            let node = ExplicitNode {
                head: t.get_head_symbol().protect(),
                children,
            };

            if node.children.iter().all(|c| c.is_compressed()) {
                Compressed(t.protect())
            } else {
                Explicit(node)
            }
        }
    }

    /// Used to check if a subterm is duplicated, for example "times(s(x), y) =
    /// plus(y, times(x,y))" is duplicating.
    pub(crate) fn contains_duplicate_var_references(&self) -> bool {
        let references = self.get_all_var_references();
        let mut seen = HashSet::new();

        for r in references {
            if seen.contains(&r) {
                return true;
            }
            seen.insert(r);
        }

        false
    }

    /// Get all positions to variables in the left hand side.
    fn get_all_var_references(&self) -> Vec<ExplicitPosition> {
        let mut result = vec![];
        match self {
            Explicit(en) => {
                for n in &en.children {
                    result.extend_from_slice(&n.get_all_var_references());
                }
            }
            Compressed(_) => {}
            Variable(ep) => {
                result.push(ep.clone());
            }
        }

        result
    }

    /// Returns true iff this tree is compressed.
    fn is_compressed(&self) -> bool {
        matches!(self, Compressed(_))
    }
}

/// Create a mapping of variables to their position in the given term
pub fn create_var_map(t: &ATerm) -> HashMap<DataVariable, ExplicitPosition> {
    let mut result = HashMap::new();

    for (term, position) in PositionIterator::new(t.copy()) {
        if is_data_variable(&term) {
            result.insert(term.protect().into(), position.clone());
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use ahash::AHashSet;
    use mcrl2::aterm::apply;
    use mcrl2::aterm::TermPool;

    /// Converts a slice of static strings into a set of owned strings
    ///
    /// example:
    ///     make_var_map(["x"])
    fn var_map(vars: &[&str]) -> AHashSet<String> {
        AHashSet::from_iter(vars.iter().map(|x| String::from(*x)))
    }

    /// Convert terms in variables to a [DataVariable].
    pub fn convert_variables(tp: &mut TermPool, t: &ATerm, variables: &AHashSet<String>) -> ATerm {
        apply(tp, t, &|tp, arg| {
            if variables.contains(arg.get_head_symbol().name()) {
                // Convert a constant variable, for example 'x', into an untyped variable.
                Some(DataVariable::new(tp, &arg.get_head_symbol().name()).into())
            } else {
                None
            }
        })
    }

    #[test]
    fn test_constant() {
        let mut tp = TermPool::new();
        let t = tp.from_string("a").unwrap();

        let map = HashMap::new();
        let sctt = SemiCompressedTermTree::from_term(&t, &map);
        assert_eq!(sctt, Compressed(t));
    }

    #[test]
    fn test_compressible() {
        let mut tp = TermPool::new();
        let t = tp.from_string("f(a,a)").unwrap();

        let map = HashMap::new();
        let sctt = SemiCompressedTermTree::from_term(&t, &&map);
        assert_eq!(sctt, Compressed(t));
    }

    #[test]
    fn test_not_compressible() {
        let mut tp = TermPool::new();
        let t = {
            let tmp = tp.from_string("f(x,x)").unwrap();
            convert_variables(&mut tp, &tmp, &var_map(&["x"]))
        };

        let mut map = HashMap::new();
        map.insert(DataVariable::new(&mut tp, "x"), ExplicitPosition::new(&[2]));

        let sctt = SemiCompressedTermTree::from_term(&t, &map);

        let en = Explicit(ExplicitNode {
            head: tp.create_symbol("f", 2),
            children: vec![
                Variable(ExplicitPosition::new(&[2])), // Note that both point to the second occurence of x.
                Variable(ExplicitPosition::new(&[2])),
            ],
        });

        assert_eq!(sctt, en);
    }

    #[test]
    fn test_partly_compressible() {
        let mut tp = TermPool::new();
        let t = {
            let tmp = tp.from_string("f(f(a,a),x)").unwrap();
            convert_variables(&mut tp, &tmp, &var_map(&["x"]))
        };
        let compressible = tp.from_string("f(a,a)").unwrap();

        // Make a variable map with only x@2.
        let mut map = HashMap::new();
        map.insert(DataVariable::new(&mut tp, "x"), ExplicitPosition::new(&[2]));

        let sctt = SemiCompressedTermTree::from_term(&t, &map);
        let en = Explicit(ExplicitNode {
            head: tp.create_symbol("f", 2),
            children: vec![Compressed(compressible), Variable(ExplicitPosition::new(&[2]))],
        });
        assert_eq!(sctt, en);
    }

    #[test]
    fn test_evaluation() {
        let mut tp = TermPool::new();
        let t_rhs = {
            let tmp = tp.from_string("f(f(a,a),x)").unwrap();
            convert_variables(&mut tp, &tmp, &var_map(&["x"]))
        };
        let t_lhs = tp.from_string("g(b)").unwrap();

        // Make a variable map with only x@1.
        let mut map = HashMap::new();
        map.insert(DataVariable::new(&mut tp, "x"), ExplicitPosition::new(&[1]));

        let sctt = SemiCompressedTermTree::from_term(&t_rhs, &map);

        let t_expected = tp.from_string("f(f(a,a),b)").unwrap();
        assert_eq!(sctt.evaluate(&t_lhs, &mut tp), t_expected);
    }

    #[test]
    fn test_create_varmap() {
        let mut tp = TermPool::new();
        let t = {
            let tmp = tp.from_string("f(x,x)").unwrap();
            convert_variables(&mut tp, &tmp, &AHashSet::from([String::from("x")]))
        };
        let x = DataVariable::new(&mut tp, "x");

        let map = create_var_map(&t);
        assert!(map.contains_key(&x));
    }

    #[test]
    fn test_is_duplicating() {
        let mut tp = TermPool::new();
        let t_rhs = {
            let tmp = tp.from_string("f(x,x)").unwrap();
            convert_variables(&mut tp, &tmp, &AHashSet::from([String::from("x")]))
        };

        // Make a variable map with only x@1.
        let mut map = HashMap::new();
        map.insert(DataVariable::new(&mut tp, "x"), ExplicitPosition::new(&[1]));

        let sctt = SemiCompressedTermTree::from_term(&t_rhs, &map);
        assert!(sctt.contains_duplicate_var_references(), "This sctt is duplicating");
    }
}
