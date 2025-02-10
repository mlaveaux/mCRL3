use std::fmt;

#[derive(Debug)]
pub struct Mcrl2Specification {
    pub map: Vec<IdsDecl>,
}

#[derive(Debug)]
pub struct IdsDecl {
    pub identifiers: Vec<String>,
    pub sort: SortExpression,
    pub span: Span,
}

#[derive(Debug)]
pub enum SortExpression {
    Product {
        lhs: Box<SortExpression>,
        rhs: Box<SortExpression>,
    },
    Function {
        domain: Box<SortExpression>,
        range: Box<SortExpression>,
    },
    Reference(String),
    Simple(Sort),
    Complex(ComplexSort, Box<SortExpression>),
}

#[derive(Debug)]
pub enum Sort {
    Bool,
    Pos,
    Int,
    Nat,
    Real,
}

#[derive(Debug)]
pub enum ComplexSort {
    List,
    Set,
    FSet,
    FBag,
}

pub struct Span {
    start: usize,
    end: usize,
}

impl From<pest::Span<'_>> for Span {
    fn from(span: pest::Span) -> Self {
        Span {
            start: span.start(),
            end: span.end(),
        }
    }
}

pub fn print_location(input: &str, span: &Span) {
    input.lines().enumerate().fold(span.start, |current, (number, line)| {
        if current < line.len() {
            println!("ln {number}, col {}", span.start - current);
        }

        current - line.len()
    });
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

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

impl fmt::Display for Mcrl2Specification {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for decl in &self.map {
            writeln!(f, "{}", decl)?;
        }
        Ok(())
    }
}

impl fmt::Display for IdsDecl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} : {}", self.identifiers.join(", "), self.sort)
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
        }
    }
}