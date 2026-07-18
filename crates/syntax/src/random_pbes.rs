use rand::Rng;
use rand::RngExt;
use rand::seq::IndexedRandom;

use crate::DataExpr;
use crate::DataExprBinaryOp;
use crate::DataExprKind;
use crate::FixedPointOperator;
use crate::IdDecl;
use crate::PbesEquation;
use crate::PbesExpr;
use crate::PbesExprBinaryOp;
use crate::PbesExprKind;
use crate::PropVarDecl;
use crate::PropVarInst;
use crate::Quantifier;
use crate::Sort;
use crate::SortExpressionKind;
use crate::Span;
use crate::UntypedPbes;
use crate::random_boolean_data_expression;
use crate::random_integer_data_expression;

const PRED_INTS: &[&str] = &["m", "n"];
const PRED_BOOLS: &[&str] = &["b", "c"];
const QUANT_INTS: &[&str] = &["t", "u", "v", "w"];

/// Parameters held constant throughout the random PBES generation.
struct PbesGenConfig<'a> {
    /// The predicate variables available for instantiation in leaves.
    predicate_vars: &'a [PredVar],

    /// Whether quantifiers may be generated.
    use_quantifiers: bool,

    /// Probability that a leaf is a predicate variable instantiation rather than
    /// a `val(...)` atom.
    propvar_probability: f64,
}

/// Generates a random PBES.
///
/// `atom_count` and `propvar_count` together control the expression size: their sum determines the
/// recursion depth, and their ratio determines how often a leaf is a predicate variable instantiation
/// versus a `val(...)` atom.
pub fn random_pbes<R: Rng>(
    rng: &mut R,
    equation_count: usize,
    atom_count: usize,
    propvar_count: usize,
    use_quantifiers: bool,
    use_integers: bool,
) -> UntypedPbes {
    let pred_vars: Vec<PredVar> = (0..equation_count)
        .map(|i| make_pred_var(rng, i, use_integers))
        .collect();

    let total = (atom_count + propvar_count).max(1);
    let propvar_prob = propvar_count as f64 / total as f64;
    let depth = total.ilog2() as usize + 1;

    let config = PbesGenConfig {
        predicate_vars: &pred_vars,
        use_quantifiers,
        propvar_probability: propvar_prob,
    };

    let mut equations = Vec::new();
    for pv in &pred_vars {
        let freevars = pv.expr_freevars();
        let formula = random_pbes_expr(rng, depth, &freevars, &config, false);
        let operator = if rng.random_bool(0.5) {
            FixedPointOperator::Least
        } else {
            FixedPointOperator::Greatest
        };
        equations.push(PbesEquation::new(operator, pv.to_decl(), formula));
    }

    let first = &pred_vars[0];
    let init_args: Vec<DataExpr> = first
        .params
        .iter()
        .map(|p| {
            if is_bool_var(p) {
                DataExprKind::Bool(true).into()
            } else {
                DataExprKind::Number("0".to_string()).into()
            }
        })
        .collect();
    let init = PropVarInst::new(first.name.clone(), init_args);

    UntypedPbes {
        data_specification: Default::default(),
        global_variables: Vec::new(),
        equations,
        init,
    }
}

fn random_leaf<R: Rng>(rng: &mut R, freevars: &[IdDecl], config: &PbesGenConfig, negated: bool) -> PbesExpr {
    if !config.predicate_vars.is_empty() && rng.random_bool(config.propvar_probability) {
        let pv = config.predicate_vars.choose(rng).unwrap();
        let args = pv
            .params
            .iter()
            .map(|p| {
                if is_bool_var(p) {
                    random_boolean_data_expression(rng, freevars)
                } else {
                    random_integer_data_expression(rng, freevars)
                }
            })
            .collect();
        let inst = PropVarInst::new(pv.name.clone(), args);
        if negated {
            PbesExprKind::Negation(Box::new(PbesExprKind::PropVarInst(inst).into())).into()
        } else {
            PbesExprKind::PropVarInst(inst).into()
        }
    } else {
        PbesExprKind::DataValExpr(random_boolean_data_expression(rng, freevars)).into()
    }
}

/// Generates a random PBES expression with the given parameters.  
///
/// `depth` controls the maximum depth of the generated expression, and
/// `propvar_probability` controls how likely a leaf is to be a predicate variable
/// instantiation versus a `val(...)` atom.  If `use_quantifiers` is false, no
/// quantifiers will be generated.  If `negated` is true, the top-level polarity
/// is negative, which biases the generator to produce more negations and
/// implications, which flip polarity.
fn random_pbes_expr<R: Rng>(
    rng: &mut R,
    depth: usize,
    freevars: &[IdDecl],
    config: &PbesGenConfig,
    negated: bool,
) -> PbesExpr {
    if depth == 0 {
        return random_leaf(rng, freevars, config, negated);
    }

    // Binary operators are over-represented to bias toward non-trivial trees.
    let op_table: &[u8] = if config.use_quantifiers {
        &[0, 1, 2, 3, 0, 1, 2, 3, 4, 5]
    } else {
        &[0, 1, 2, 3]
    };
    let op = *op_table.choose(rng).unwrap();

    match op {
        0 => {
            let inner = random_pbes_expr(rng, depth - 1, freevars, config, !negated);
            PbesExprKind::Negation(Box::new(inner)).into()
        }
        1 => {
            let l = random_pbes_expr(rng, depth - 1, freevars, config, negated);
            let r = random_pbes_expr(rng, depth - 1, freevars, config, negated);
            PbesExprKind::Binary {
                op: PbesExprBinaryOp::Conjunction,
                lhs: Box::new(l),
                rhs: Box::new(r),
            }
            .into()
        }
        2 => {
            let l = random_pbes_expr(rng, depth - 1, freevars, config, negated);
            let r = random_pbes_expr(rng, depth - 1, freevars, config, negated);
            PbesExprKind::Binary {
                op: PbesExprBinaryOp::Disjunction,
                lhs: Box::new(l),
                rhs: Box::new(r),
            }
            .into()
        }
        3 => {
            // Antecedent flips polarity for monotonicity.
            let l = random_pbes_expr(rng, depth - 1, freevars, config, !negated);
            let r = random_pbes_expr(rng, depth - 1, freevars, config, negated);
            PbesExprKind::Binary {
                op: PbesExprBinaryOp::Implies,
                lhs: Box::new(l),
                rhs: Box::new(r),
            }
            .into()
        }
        4 => random_quantifier(rng, Quantifier::Forall, depth - 1, freevars, config, negated),
        5 => random_quantifier(rng, Quantifier::Exists, depth - 1, freevars, config, negated),
        _ => unreachable!(),
    }
}

/// Generates a random quantifier expression.  The quantifier variable is always
/// of type Nat and is artificially bounded (e.g. `forall t. t < 3 => body`) to
/// ensure termination of the generator.  The variable is added to `freevars`
/// when generating the body, so it may be used there.  If `negated` is true,
/// the quantifier is generated with negative polarity, which biases the
/// generator to produce more negations and implications, which flip polarity.
fn random_quantifier<R: Rng>(
    rng: &mut R,
    quantifier: Quantifier,
    depth: usize,
    freevars: &[IdDecl],
    config: &PbesGenConfig,
    negated: bool,
) -> PbesExpr {
    let available: Vec<&str> = QUANT_INTS
        .iter()
        .filter(|&&q| !freevars.iter().any(|fv| fv.identifier == q))
        .copied()
        .collect();

    if available.is_empty() {
        return random_leaf(rng, freevars, config, negated);
    }

    let var_name = (*available.choose(rng).expect("available is non-empty")).to_string();
    let var_decl = IdDecl::new(var_name.clone(), SortExpressionKind::Simple(Sort::Nat).into(), Span::default());

    let mut new_freevars = freevars.to_vec();
    new_freevars.push(as_expr_decl(&var_name));

    let body = random_pbes_expr(rng, depth, &new_freevars, config, negated);

    // Bound the quantifier variable to ensure termination: forall t. t < 3 => body  /  exists t. t < 3 && body
    let bound = PbesExprKind::DataValExpr(
        DataExprKind::Binary {
            op: DataExprBinaryOp::LessThan,
            lhs: Box::new(DataExprKind::Id(var_name).into()),
            rhs: Box::new(DataExprKind::Number("3".to_string()).into()),
        }
        .into(),
    )
    .into();
    let bounded_body = match quantifier {
        Quantifier::Forall => PbesExprKind::Binary {
            op: PbesExprBinaryOp::Implies,
            lhs: Box::new(bound),
            rhs: Box::new(body),
        }
        .into(),
        Quantifier::Exists => PbesExprKind::Binary {
            op: PbesExprBinaryOp::Conjunction,
            lhs: Box::new(bound),
            rhs: Box::new(body),
        }
        .into(),
    };

    PbesExprKind::Quantifier {
        quantifier,
        variables: vec![var_decl],
        body: Box::new(bounded_body),
    }
    .into()
}

fn is_bool_var(name: &str) -> bool {
    PRED_BOOLS.contains(&name)
}

fn as_expr_decl(name: &str) -> IdDecl {
    let sort = if is_bool_var(name) { Sort::Bool } else { Sort::Nat };
    IdDecl::new(name.to_string(), SortExpressionKind::Simple(sort).into(), Span::default())
}

struct PredVar {
    name: String,
    params: Vec<String>,
}

impl PredVar {
    fn to_decl(&self) -> PropVarDecl {
        let params = self
            .params
            .iter()
            .map(|p| {
                let sort = if is_bool_var(p) { Sort::Bool } else { Sort::Nat };
                IdDecl::new(p.clone(), SortExpressionKind::Simple(sort).into(), Span::default())
            })
            .collect();
        PropVarDecl::new(self.name.clone(), params)
    }

    fn expr_freevars(&self) -> Vec<IdDecl> {
        self.params.iter().map(|p| as_expr_decl(p)).collect()
    }
}

fn make_pred_var<R: Rng>(rng: &mut R, index: usize, use_integers: bool) -> PredVar {
    let size = rng.random_range(0..=2usize);
    let mut pool: Vec<&str> = if use_integers {
        PRED_INTS.iter().chain(PRED_BOOLS.iter()).copied().collect()
    } else {
        PRED_BOOLS.to_vec()
    };
    let mut params = Vec::new();
    for _ in 0..size {
        if pool.is_empty() {
            break;
        }
        let idx = rng.random_range(0..pool.len());
        params.push(pool.remove(idx).to_string());
    }
    PredVar {
        name: format!("X{index}"),
        params,
    }
}
