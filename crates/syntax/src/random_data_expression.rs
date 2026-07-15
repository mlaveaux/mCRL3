use rand::Rng;
use rand::prelude::IteratorRandom;

use crate::DataExpr;
use crate::DataExprBinaryOp;
use crate::IdDecl;
use crate::Sort;
use crate::SortExpression;

/// Generates a random boolean data expression from the given variable list.
pub fn random_boolean_data_expression<R: Rng, Id>(rng: &mut R, variables: &[IdDecl<Id>]) -> DataExpr {
    let integers: Vec<&IdDecl<Id>> = variables
        .iter()
        .filter(|v| matches!(&v.sort, SortExpression::Simple(s) if matches!(s, Sort::Int | Sort::Nat | Sort::Pos)))
        .collect();
    let booleans: Vec<&IdDecl<Id>> = variables
        .iter()
        .filter(|v| matches!(&v.sort, SortExpression::Simple(Sort::Bool)))
        .collect();

    let mut candidates: Vec<DataExpr> = booleans.iter().map(|v| DataExpr::Id(v.identifier.clone())).collect();

    for m in &integers {
        let mv = DataExpr::Id(m.identifier.clone());
        candidates.push(DataExpr::Binary {
            op: DataExprBinaryOp::GreaterThan,
            lhs: Box::new(mv.clone()),
            rhs: Box::new(DataExpr::Number("0".to_string())),
        });
        candidates.push(DataExpr::Binary {
            op: DataExprBinaryOp::GreaterThan,
            lhs: Box::new(mv.clone()),
            rhs: Box::new(DataExpr::Number("1".to_string())),
        });
        candidates.push(DataExpr::Binary {
            op: DataExprBinaryOp::LessThan,
            lhs: Box::new(mv.clone()),
            rhs: Box::new(DataExpr::Number("2".to_string())),
        });
        candidates.push(DataExpr::Binary {
            op: DataExprBinaryOp::LessThan,
            lhs: Box::new(mv.clone()),
            rhs: Box::new(DataExpr::Number("3".to_string())),
        });
        for n in &integers {
            candidates.push(DataExpr::Binary {
                op: DataExprBinaryOp::Equal,
                lhs: Box::new(mv.clone()),
                rhs: Box::new(DataExpr::Id(n.identifier.clone())),
            });
        }
    }

    candidates.push(DataExpr::Bool(true));
    candidates.push(DataExpr::Bool(false));

    candidates.into_iter().choose(rng).unwrap()
}

/// Generates a random integer data expression from the given variable list.
pub fn random_integer_data_expression<R: Rng, Id>(rng: &mut R, variables: &[IdDecl<Id>]) -> DataExpr {
    let integers: Vec<&IdDecl<Id>> = variables
        .iter()
        .filter(|v| matches!(&v.sort, SortExpression::Simple(s) if matches!(s, Sort::Int | Sort::Nat | Sort::Pos)))
        .collect();

    let extras = [DataExpr::Number("1".to_string()), DataExpr::Number("2".to_string())];
    let rhs_operands: Vec<DataExpr> = integers
        .iter()
        .map(|v| DataExpr::Id(v.identifier.clone()))
        .chain(extras)
        .collect();

    let mut candidates: Vec<DataExpr> = integers.iter().map(|v| DataExpr::Id(v.identifier.clone())).collect();

    for m in &integers {
        let mv = DataExpr::Id(m.identifier.clone());
        for n in &rhs_operands {
            candidates.push(DataExpr::Binary {
                op: DataExprBinaryOp::Add,
                lhs: Box::new(mv.clone()),
                rhs: Box::new(n.clone()),
            });
            candidates.push(DataExpr::Binary {
                op: DataExprBinaryOp::Subtract,
                lhs: Box::new(mv.clone()),
                rhs: Box::new(n.clone()),
            });
        }
    }

    candidates.push(DataExpr::Number("0".to_string()));
    candidates.push(DataExpr::Number("1".to_string()));

    candidates.into_iter().choose(rng).unwrap()
}
