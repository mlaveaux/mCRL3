use rand::Rng;
use rand::RngExt;
use rand::seq::IndexedRandom;

use crate::ActDecl;
use crate::Assignment;
use crate::CommExpr;
use crate::DataExprBinaryOp;
use crate::DataExprKind;
use crate::IdDecl;
use crate::MultiActionLabel;
use crate::ProcDecl;
use crate::ProcExprBinaryOp;
use crate::ProcessExpr;
use crate::Rename;
use crate::Sort;
use crate::SortExpression;
use crate::Span;
use crate::UntypedDataSpecification;
use crate::UntypedProcessSpecification;
use crate::random_boolean_data_expression;
use crate::random_integer_data_expression;

/// Generates a random linear process specification as an AST.
///
/// Models a finite-state machine: `num_states` states (encoded as a `Nat` parameter `s`),
/// `num_actions` label-only actions (a0, a1, …), each (state, action) pair enabled with
/// probability `transition_prob`, and the target state chosen uniformly. The returned
/// specification is accepted by `txt2lps` when printed (no linearization step required).
pub fn random_lps<R: Rng>(
    rng: &mut R,
    num_states: usize,
    num_actions: usize,
    transition_prob: f64,
) -> UntypedProcessSpecification {
    assert!(num_states >= 1 && num_actions >= 1);

    let action_names: Vec<String> = (0..num_actions).map(|i| format!("a{i}")).collect();

    let action_declarations = action_names
        .iter()
        .map(|name| ActDecl {
            identifier: name.clone(),
            args: Vec::new(),
            span: Span::default(),
        })
        .collect();

    let s_param = IdDecl::new("s".to_string(), SortExpression::Simple(Sort::Nat), Span::default());

    let mut summands: Vec<ProcessExpr> = Vec::new();
    for from in 0..num_states {
        for act in &action_names {
            if rng.random_bool(transition_prob) {
                let to = rng.random_range(0..num_states);
                let condition = DataExprKind::Binary {
                    op: DataExprBinaryOp::Equal,
                    lhs: Box::new(DataExprKind::Id("s".to_string()).into()),
                    rhs: Box::new(DataExprKind::Number(from.to_string()).into()),
                }
                .into();
                let seq = ProcessExpr::Binary {
                    op: ProcExprBinaryOp::Sequence,
                    lhs: Box::new(ProcessExpr::Action(act.clone(), Vec::new())),
                    rhs: Box::new(ProcessExpr::Id(
                        "P".to_string(),
                        vec![Assignment {
                            identifier: "s".to_string(),
                            expr: DataExprKind::Number(to.to_string()).into(),
                        }],
                    )),
                };
                summands.push(ProcessExpr::Condition {
                    condition,
                    then: Box::new(seq),
                    else_: None,
                });
            }
        }
    }

    // Keep the spec syntactically valid when no transitions are generated.
    if summands.is_empty() {
        summands.push(ProcessExpr::Delta);
    }

    let body = summands
        .into_iter()
        .reduce(|acc, s| ProcessExpr::Binary {
            op: ProcExprBinaryOp::Choice,
            lhs: Box::new(acc),
            rhs: Box::new(s),
        })
        .expect("summands is non-empty");

    let process_declarations = vec![ProcDecl {
        identifier: "P".to_string(),
        params: vec![s_param],
        body,
        span: Span::default(),
    }];

    let init_state = rng.random_range(0..num_states);
    let init = ProcessExpr::Id(
        "P".to_string(),
        vec![Assignment {
            identifier: "s".to_string(),
            expr: DataExprKind::Number(init_state.to_string()).into(),
        }],
    );

    UntypedProcessSpecification {
        data_specification: UntypedDataSpecification::default(),
        global_variables: Vec::new(),
        action_declarations,
        process_declarations,
        init: Some(init),
    }
}

const ACTIONS: &[&str] = &["a", "b", "c", "d"];
const PROC_NAMES: &[&str] = &["P", "Q", "R"];

// Sum-variable names are kept distinct from process parameter names (b, c, m, n).
const SUM_VARS: &[&str] = &["s1", "s2", "s3"];

fn id_decl(name: &str, sort: Sort) -> IdDecl {
    IdDecl::new(name.to_string(), SortExpression::Simple(sort), Span::default())
}

fn is_bool(decl: &IdDecl) -> bool {
    matches!(&decl.sort, SortExpression::Simple(Sort::Bool))
}

struct ProcVar {
    name: String,
    params: Vec<IdDecl>,
}

fn random_proc_var<R: Rng>(rng: &mut R, index: usize, use_integers: bool) -> ProcVar {
    let size = rng.random_range(0..=2usize);
    let mut pool: Vec<IdDecl> = vec![id_decl("b", Sort::Bool), id_decl("c", Sort::Bool)];
    if use_integers {
        pool.push(id_decl("m", Sort::Nat));
        pool.push(id_decl("n", Sort::Nat));
    }
    let mut params = Vec::new();
    for _ in 0..size {
        if pool.is_empty() {
            break;
        }
        let idx = rng.random_range(0..pool.len());
        params.push(pool.remove(idx));
    }
    ProcVar {
        name: PROC_NAMES[index].to_string(),
        params,
    }
}

fn random_process_instance<R: Rng>(rng: &mut R, pv: &ProcVar, freevars: &[IdDecl]) -> ProcessExpr {
    let assignments = pv
        .params
        .iter()
        .map(|p| {
            let expr = if is_bool(p) {
                random_boolean_data_expression(rng, freevars)
            } else {
                random_integer_data_expression(rng, freevars)
            };
            Assignment {
                identifier: p.identifier.clone(),
                expr,
            }
        })
        .collect();
    ProcessExpr::Id(pv.name.clone(), assignments)
}

fn random_leaf<R: Rng>(
    rng: &mut R,
    freevars: &[IdDecl],
    actions: &[&str],
    proc_vars: &[ProcVar],
    is_guarded: bool,
) -> ProcessExpr {
    // Build a weighted table: Action has high weight, Delta/Tau low, ProcessInstance medium.
    let mut table: Vec<usize> = vec![0, 1]; // Delta=0, Tau=1
    if !actions.is_empty() {
        table.extend(std::iter::repeat_n(2, 8));
    }

    if !proc_vars.is_empty() && !is_guarded {
        table.extend(std::iter::repeat_n(3, 2)); // ProcessInstance=3
    }

    match *table.choose(rng).expect("table always contains at least Delta and Tau") {
        0 => ProcessExpr::Delta,
        1 => ProcessExpr::Tau,
        2 => ProcessExpr::Action(
            (*actions.choose(rng).expect("actions is non-empty")).to_string(),
            Vec::new(),
        ),
        3 => {
            let pv = proc_vars.choose(rng).expect("proc_vars is non-empty");
            random_process_instance(rng, pv, freevars)
        }
        _ => unreachable!(),
    }
}

fn random_process_expr<R: Rng>(
    rng: &mut R,
    depth: usize,
    freevars: &[IdDecl],
    actions: &[&str],
    proc_vars: &[ProcVar],
    is_guarded: bool,
) -> ProcessExpr {
    if depth == 0 {
        return random_leaf(rng, freevars, actions, proc_vars, is_guarded);
    }

    // op: 0=leaf, 1=sum, 2=if-then, 3=if-then-else, 4=choice, 5=seq
    // seq is over-represented to bias toward action-guarded continuations.
    let op_table: &[usize] = &[0, 0, 1, 2, 3, 4, 4, 5, 5, 5];
    match *op_table.choose(rng).expect("op_table is a non-empty constant") {
        0 => random_leaf(rng, freevars, actions, proc_vars, is_guarded),
        1 => {
            // Sum: bind a fresh variable chosen from SUM_VARS to avoid capture.
            let var_name = SUM_VARS
                .iter()
                .find(|&&n| !freevars.iter().any(|v| v.identifier == n))
                .copied();
            match var_name {
                None => random_leaf(rng, freevars, actions, proc_vars, is_guarded),
                Some(name) => {
                    let sort = if rng.random_bool(0.5) { Sort::Bool } else { Sort::Nat };
                    let var = id_decl(name, sort);
                    let mut new_vars = freevars.to_vec();
                    new_vars.push(var.clone());
                    let body = random_process_expr(rng, depth - 1, &new_vars, actions, proc_vars, is_guarded);
                    ProcessExpr::Sum {
                        variables: vec![var],
                        operand: Box::new(body),
                    }
                }
            }
        }
        2 => {
            // IfThen: condition -> body
            let cond = random_boolean_data_expression(rng, freevars);
            let body = random_process_expr(rng, depth - 1, freevars, actions, proc_vars, is_guarded);
            ProcessExpr::Condition {
                condition: cond,
                then: Box::new(body),
                else_: None,
            }
        }
        3 => {
            // IfThenElse: condition -> x <> y
            let cond = random_boolean_data_expression(rng, freevars);
            let then = random_process_expr(rng, depth - 1, freevars, actions, proc_vars, is_guarded);
            let else_ = random_process_expr(rng, depth - 1, freevars, actions, proc_vars, is_guarded);
            ProcessExpr::Condition {
                condition: cond,
                then: Box::new(then),
                else_: Some(Box::new(else_)),
            }
        }
        4 => {
            // Choice: lhs + rhs (each branch must independently satisfy guardedness)
            let lhs = random_process_expr(rng, depth - 1, freevars, actions, proc_vars, is_guarded);
            let rhs = random_process_expr(rng, depth - 1, freevars, actions, proc_vars, is_guarded);
            ProcessExpr::Binary {
                op: ProcExprBinaryOp::Choice,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }
        }
        5 => {
            // Seq: emit an action first, then an (unguarded) continuation.
            // The explicit action satisfies the guard, so the rhs may reference proc instances.
            let action = (*actions.choose(rng).expect("actions is non-empty")).to_string();
            let lhs = ProcessExpr::Action(action, Vec::new());
            let rhs = random_process_expr(rng, depth - 1, freevars, actions, proc_vars, false);
            ProcessExpr::Binary {
                op: ProcExprBinaryOp::Sequence,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }
        }
        _ => unreachable!(),
    }
}

fn apply_wrapper<R: Rng>(rng: &mut R, actions: &[&str], expr: ProcessExpr) -> ProcessExpr {
    match rng.random_range(0..5usize) {
        0 => {
            let a = (*actions.choose(rng).expect("actions is non-empty")).to_string();
            ProcessExpr::Hide {
                actions: vec![a],
                operand: Box::new(expr),
            }
        }
        1 => {
            let a = (*actions.choose(rng).expect("actions is non-empty")).to_string();
            ProcessExpr::Block {
                actions: vec![a],
                operand: Box::new(expr),
            }
        }
        2 if actions.len() >= 2 => {
            let mut pool = actions.to_vec();
            let ai = rng.random_range(0..pool.len());
            let from = pool.remove(ai).to_string();
            let to = (*pool
                .choose(rng)
                .expect("pool has at least one element after removing from"))
            .to_string();
            ProcessExpr::Rename {
                renames: vec![Rename { from, to }],
                operand: Box::new(expr),
            }
        }
        3 if actions.len() >= 3 => {
            // Comm: a | b -> c  (mCRL2 synchronisation mapping)
            let mut pool: Vec<String> = actions.iter().map(|&s| s.to_string()).collect();
            let ai = rng.random_range(0..pool.len());
            let a = pool.remove(ai);
            let bi = rng.random_range(0..pool.len());
            let b = pool.remove(bi);
            let c = pool
                .choose(rng)
                .expect("pool has at least one element after removing a and b")
                .clone();
            ProcessExpr::Comm {
                comm: vec![CommExpr::new(MultiActionLabel::new(vec![a, b]), c)],
                operand: Box::new(expr),
            }
        }
        4 => {
            let mut labels: Vec<MultiActionLabel> = (0..5)
                .map(|_| {
                    let size = rng.random_range(1..=2usize);
                    let mut acts: Vec<String> = (0..size)
                        .map(|_| (*actions.choose(rng).expect("actions is non-empty")).to_string())
                        .collect();
                    acts.sort();
                    acts.dedup();
                    MultiActionLabel::new(acts)
                })
                .collect();
            labels.sort();
            labels.dedup();
            ProcessExpr::Allow {
                actions: labels,
                operand: Box::new(expr),
            }
        }
        _ => expr, // guard failures from arms 2/3 fall here
    }
}

fn random_parallel_init<R: Rng>(
    rng: &mut R,
    actions: &[&str],
    mut procs: Vec<ProcessExpr>,
    wrapper_count: usize,
) -> ProcessExpr {
    // Fold all instances into a single parallel composition tree.
    while procs.len() > 1 {
        let n = procs.len();
        let j = rng.random_range(1..n);
        let p = procs.remove(j);
        let q = procs.remove(0);
        procs.push(ProcessExpr::Binary {
            op: ProcExprBinaryOp::Parallel,
            lhs: Box::new(q),
            rhs: Box::new(p),
        });
    }
    let mut result = procs.remove(0);
    for _ in 0..wrapper_count {
        result = apply_wrapper(rng, actions, result);
    }
    result
}

/// Generates a random mCRL2 process specification.
///
/// `equation_count` controls how many process equations are produced (capped at 3).
/// `depth` controls the maximum nesting depth of each process body.
/// `use_integers` adds `Nat`-typed parameters alongside `Bool` ones.
pub fn make_process_specification<R: Rng>(
    rng: &mut R,
    equation_count: usize,
    depth: usize,
    use_integers: bool,
) -> UntypedProcessSpecification {
    let count = equation_count.min(PROC_NAMES.len());
    let proc_vars: Vec<ProcVar> = (0..count).map(|i| random_proc_var(rng, i, use_integers)).collect();

    let action_declarations = ACTIONS
        .iter()
        .map(|&name| ActDecl {
            identifier: name.to_string(),
            args: Vec::new(),
            span: Span::default(),
        })
        .collect();

    let process_declarations: Vec<ProcDecl> = proc_vars
        .iter()
        .map(|pv| {
            let body = random_process_expr(rng, depth, &pv.params, ACTIONS, &proc_vars, true);
            ProcDecl {
                identifier: pv.name.clone(),
                params: pv.params.clone(),
                body,
                span: Span::default(),
            }
        })
        .collect();

    let instances: Vec<ProcessExpr> = proc_vars
        .iter()
        .map(|pv| {
            let assignments = pv
                .params
                .iter()
                .map(|p| {
                    let expr = if is_bool(p) {
                        DataExprKind::Bool(rng.random_bool(0.5)).into()
                    } else {
                        DataExprKind::Number(rng.random_range(0..=2u32).to_string()).into()
                    };
                    Assignment {
                        identifier: p.identifier.clone(),
                        expr,
                    }
                })
                .collect();
            ProcessExpr::Id(pv.name.clone(), assignments)
        })
        .collect();

    let init = if instances.is_empty() {
        ProcessExpr::Delta
    } else {
        let wrapper_count = rng.random_range(0..=5usize);
        random_parallel_init(rng, ACTIONS, instances, wrapper_count)
    };

    UntypedProcessSpecification {
        data_specification: UntypedDataSpecification::default(),
        global_variables: Vec::new(),
        action_declarations,
        process_declarations,
        init: Some(init),
    }
}
