use std::fmt;

use itertools::Itertools;

use crate::ActFrm;
use crate::ActFrmOp;
use crate::Action;
use crate::Assignment;
use crate::ComplexSort;
use crate::ConstructorDecl;
use crate::DataExpr;
use crate::DataExprBinaryOp;
use crate::EqnDecl;
use crate::EqnSpec;
use crate::IdDecl;
use crate::ModalityOperator;
use crate::MultiAction;
use crate::Quantifier;
use crate::RegFrm;
use crate::Sort;
use crate::SortDecl;
use crate::SortExpression;
use crate::Span;
use crate::StateFrm;
use crate::StateFrmOp;
use crate::UntypedDataSpecification;
use crate::UntypedProcessSpecification;
use crate::UntypedStateFrmSpec;
use crate::VarDecl;

/// Prints location information for a span in the source.
pub fn print_location(input: &str, span: &Span) {
    input.lines().enumerate().fold(span.start, |current, (number, line)| {
        if current < line.len() {
            println!("ln {number}, col {}", span.start - current);
        }
        current - line.len()
    });
}

// Display implementations
impl fmt::Display for Sort {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for ComplexSort {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for Assignment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} := {:?}", self.identifier, self.expr)
    }
}

impl fmt::Display for UntypedProcessSpecification {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.data_specification)?;
        Ok(())
    }
}

impl fmt::Display for UntypedDataSpecification {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for decl in &self.sort_decls {
            writeln!(f, "{}", decl)?;
        }
        for decl in &self.cons_decls {
            writeln!(f, "{}", decl)?;
        }

        writeln!(f, "map")?;
        for decl in &self.map_decls {
            writeln!(f, "   {}", decl)?;
        }

        for decl in &self.eqn_decls {
            writeln!(f, "{}", decl)?;
        }
        Ok(())
    }
}

impl fmt::Display for EqnSpec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "var")?;
        for decl in &self.variables {
            writeln!(f, "   {}", decl)?;
        }

        writeln!(f, "eqn")?;
        for decl in &self.equations {
            writeln!(f, "   {}", decl)?;
        }
        Ok(())
    }
}

impl fmt::Display for SortDecl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "sort {}", self.identifier)?;

        if let Some(expr) = &self.expr {
            write!(f, " = {}", expr)?;
        }

        Ok(())
    }
}

impl fmt::Display for VarDecl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} : {}", self.identifier, self.sort)
    }
}

impl fmt::Display for EqnDecl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.condition {
            Some(condition) => write!(f, "{} -> {} = {};", condition, self.lhs, self.rhs),
            None => write!(f, "{} = {};", self.lhs, self.rhs),
        }
    }
}

impl fmt::Display for DataExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<DataExpr>")
    }
}

impl fmt::Display for IdDecl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} : {}", self.identifier, self.sort)
    }
}

impl fmt::Display for SortExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SortExpression::Product { lhs, rhs } => write!(f, "({} # {})", lhs, rhs),
            SortExpression::Function { domain, range } => write!(f, "({} -> {})", domain, range),
            SortExpression::Reference(ident) => write!(f, "\"{}\"", ident),
            SortExpression::Simple(sort) => write!(f, "{}", sort),
            SortExpression::Complex(complex, inner) => write!(f, "{}({})", complex, inner),
            SortExpression::Struct { inner } => {
                write!(f, "{{")?;
                for (i, decl) in inner.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", decl)?;
                }
                write!(f, "}}")
            }
        }
    }
}

impl fmt::Display for UntypedStateFrmSpec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.data_specification)?;

        writeln!(f, "{}", self.formula)
    }
}

impl fmt::Display for StateFrm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StateFrm::Id(identifier, args) => {
                if args.is_empty() {
                    write!(f, "{}", identifier)
                } else {
                    write!(f, "{}({})", identifier, args.iter().format(", "))
                }
            }
            StateFrm::Negation(formula) => write!(f, "!({})", formula),
            StateFrm::Modality {
                operator,
                formula,
                expr,
            } => match operator {
                ModalityOperator::Box => write!(f, "[{}]({})", formula, expr),
                ModalityOperator::Diamond => write!(f, "<{}>({})", formula, expr),
            },
            StateFrm::Quantifier {
                quantifier,
                variables,
                body,
            } => {
                write!(f, "{} {} . ({})", quantifier, variables.iter().format(", "), body)
            }
            StateFrm::Binary { op, lhs, rhs } => {
                write!(f, "({}) {} ({})", lhs, op, rhs)
            }
            _ => write!(f, "<StateFrm>"),
        }
    }
}

impl fmt::Display for StateFrmOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StateFrmOp::Implies => write!(f, "=>"),
            StateFrmOp::Conjunction => write!(f, "&&"),
            StateFrmOp::Disjunction => write!(f, "||"),
            StateFrmOp::Addition => write!(f, "+"),
        }
    }
}

impl fmt::Display for RegFrm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RegFrm::Action(action) => write!(f, "{}", action),
            RegFrm::Iteration(body) => write!(f, "({})*", body),
            RegFrm::Plus(body) => write!(f, "({})+", body),
            RegFrm::Choice { lhs, rhs } => write!(f, "({}) + ({})", lhs, rhs),
            RegFrm::Sequence { lhs, rhs } => write!(f, "({}) . ({})", lhs, rhs),
        }
    }
}

impl fmt::Display for DataExprBinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataExprBinaryOp::At => write!(f, "."),
            DataExprBinaryOp::Concat => write!(f, "++"),
            DataExprBinaryOp::Cons => write!(f, "|>"),
            DataExprBinaryOp::Equal => write!(f, "=="),
            DataExprBinaryOp::NotEqual => write!(f, "!="),
            DataExprBinaryOp::LessThan => write!(f, "<"),
            DataExprBinaryOp::LessEqual => write!(f, "<="),
            DataExprBinaryOp::GreaterThan => write!(f, ">"),
            DataExprBinaryOp::GreaterEqual => write!(f, ">="),
            DataExprBinaryOp::Conj => write!(f, "&&"),
            DataExprBinaryOp::Disj => write!(f, "||"),
            DataExprBinaryOp::Add => write!(f, "+"),
            DataExprBinaryOp::Subtract => write!(f, "-"),
            DataExprBinaryOp::Div => write!(f, "/"),
            DataExprBinaryOp::Implies => write!(f, "=>"),
            DataExprBinaryOp::In => write!(f, "in"),
            DataExprBinaryOp::IntDiv => write!(f, "div"),
            DataExprBinaryOp::Mod => write!(f, "mod"),
            DataExprBinaryOp::Multiply => write!(f, "*"),
            DataExprBinaryOp::Snoc => write!(f, "<|"),
        }
    }
}

impl fmt::Display for ActFrm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ActFrm::False => write!(f, "false"),
            ActFrm::True => write!(f, "true"),
            ActFrm::MultAct(action) => write!(f, "{}", action),
            ActFrm::Binary { op, lhs, rhs } => {
                write!(f, "({}) {} ({})", lhs, op, rhs)
            }
            _ => write!(f, "<ActFrm>"),
        }
    }
}

impl fmt::Display for ActFrmOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ActFrmOp::Implies => write!(f, "=>"),
            ActFrmOp::Intersect => write!(f, "&&"),
            ActFrmOp::Union => write!(f, "||"),
        }
    }
}

impl fmt::Display for MultiAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.actions.is_empty() {
            write!(f, "tau")
        } else {
            write!(f, "{}", self.actions.iter().format("|"))
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.args.is_empty() {
            write!(f, "{}", self.id)
        } else {
            write!(f, "{}({})", self.id, self.args.iter().format(", "))
        }
    }
}

impl fmt::Display for Quantifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Quantifier::Exists => write!(f, "exists"),
            Quantifier::Forall => write!(f, "forall"),
        }
    }
}

impl fmt::Display for ConstructorDecl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}(", self.name)?;
        for (i, (name, sort)) in self.args.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            match name {
                Some(name) => write!(f, "{} : {}", name, sort)?,
                None => write!(f, "{}", sort)?,
            }
        }
        write!(f, ")")?;

        if let Some(projection) = &self.projection {
            write!(f, "?{}", projection)?;
        }

        Ok(())
    }
}
