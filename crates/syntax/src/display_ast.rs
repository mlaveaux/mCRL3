use std::fmt;

use crate::{Assignment, ComplexSort, ConstructorDecl, DataExpr, EqnDecl, IdDecl, Sort, SortDecl, SortExpression, Span, UntypedDataSpecification, UntypedProcessSpecification, VarDecl};

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

        writeln!(f, "glob")?;
        for decl in &self.var_decls {
            writeln!(f, "   {}", decl)?;
        }
        for decl in &self.eqn_decls {
            writeln!(f, "{}", decl)?;
        }
        Ok(())
    }
}

impl fmt::Display for SortDecl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "sort {}", self.name)?;
        for (i, decl) in self.params.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", decl)?;
        }
        writeln!(f)
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
            Some(condition) =>  write!(f, "{} -> {} = {};", condition, self.lhs, self.rhs),
            None => write!(f, "{} = {};", self.lhs, self.rhs)
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
