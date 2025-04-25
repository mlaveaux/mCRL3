use std::fmt;
use std::hash::Hash;

/// An mCRL2 specification containing declarations.
#[derive(Debug, Default, Eq, PartialEq, Hash)]
pub struct UntypedProcessSpecification {
    /// Sort declarations
    pub data_specification: UntypedDataSpecification,
    /// Global variables
    pub glob_vars: Vec<GlobVarDecl>,
    /// Action declarations
    pub act_decls: Vec<ActDecl>,
    /// Process declarations
    pub proc_decls: Vec<ProcDecl>,
    /// Initial process
    pub init: Option<ProcessExpr>,
}

#[derive(Debug, Default, Eq, PartialEq, Hash)]
pub struct UntypedDataSpecification {
    /// Sort declarations
    pub sort_decls: Vec<SortDecl>,
    /// Constructor declarations
    pub cons_decls: Vec<IdDecl>,
    /// Map declarations
    pub map_decls: Vec<IdDecl>,
    /// Variable declarations
    pub var_decls: Vec<VarDecl>,
    /// Equation declarations
    pub eqn_decls: Vec<EqnDecl>,
}

/// A declaration of an identifier with its sort.
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct IdDecl {
    /// Identifier being declared
    pub identifier: String,
    /// Sort expression for this identifier
    pub sort: SortExpression,
    /// Source location information
    pub span: Span,
}

/// Source location information.
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Span {
    /// Start position in source
    pub start: usize,
    /// End position in source
    pub end: usize,
}

impl From<pest::Span<'_>> for Span {
    fn from(span: pest::Span) -> Self {
        Span {
            start: span.start(),
            end: span.end(),
        }
    }
}

/// Expression representing a sort (type).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum SortExpression {
    /// Product of two sorts (A # B)
    Product {
        lhs: Box<SortExpression>,
        rhs: Box<SortExpression>,
    },
    /// Function sort (A -> B)
    Function {
        domain: Box<SortExpression>,
        range: Box<SortExpression>,
    },
    Struct {
        inner: Vec<ConstructorDecl>,
    },
    /// Reference to a named sort
    Reference(String),
    /// Built-in simple sort
    Simple(Sort),
    /// Parameterized complex sort
    Complex(ComplexSort, Box<SortExpression>),
}

/// Constructor declaration
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ConstructorDecl {
    pub name: String,
    pub args: Vec<(Option<String>, SortExpression)>,
    pub projection: Option<SortExpression>,
}

/// Built-in simple sorts.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Sort {
    Bool,
    Pos,
    Int,
    Nat,
    Real,
}

/// Complex (parameterized) sorts.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ComplexSort {
    List,
    Set,
    FSet,
    FBag,
    Bag,
}

/// An mCRL2 specification containing all components.
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Mcrl2Specification {}

/// Sort declaration
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct SortDecl {
    /// Sort identifier
    pub name: String,
    /// Sort parameters (if any)
    pub params: Vec<String>,
    /// Sort expression (if structured)
    pub expr: Option<SortExpression>,
    pub span: Span,
}

/// Variable declaration
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct VarDecl {
    pub identifier: String,
    pub sort: SortExpression,
    pub span: Span,
}

/// Equation declaration
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct EqnDecl {
    pub vars: Vec<VarDecl>,
    pub condition: Option<DataExpr>,
    pub lhs: DataExpr,
    pub rhs: DataExpr,
    pub span: Span,
}

/// Global variable declaration
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct GlobVarDecl {
    pub identifier: String,
    pub sort: SortExpression,
    pub init: Option<DataExpr>,
    pub span: Span,
}

/// Action declaration
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct ActDecl {
    pub name: String,
    pub args: Vec<SortExpression>,
    pub span: Span,
}

/// Process declaration
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct ProcDecl {
    pub name: String,
    pub params: Vec<VarDecl>,
    pub init: Option<DataExpr>,
    pub body: ProcessExpr,
    pub span: Span,
}

/// Data expression
#[derive(Debug, Eq, PartialEq, Hash)]
pub enum DataExpr {
    Id(String),
    Number(i64),
    Bool(bool),
    Application {
        function: Box<DataExpr>,
        arguments: Vec<DataExpr>,
    },
    List(Vec<DataExpr>),
    Set(Vec<DataExpr>),
    Bag(Vec<(DataExpr, DataExpr)>),
    Lambda {
        variables: Vec<VarDecl>,
        body: Box<DataExpr>,
    },
    Exists {
        variables: Vec<VarDecl>,
        body: Box<DataExpr>,
    },
    Forall {
        variables: Vec<VarDecl>,
        body: Box<DataExpr>,
    },
}

/// Process expression
#[derive(Debug, Eq, PartialEq, Hash)]
pub enum ProcessExpr {
    Action(String, Vec<DataExpr>),
    Delta,
    Tau,
    Sum {
        variables: Vec<VarDecl>,
        operand: Box<ProcessExpr>,
    },
    Sequence {
        left: Box<ProcessExpr>,
        right: Box<ProcessExpr>,
    },
    Choice {
        left: Box<ProcessExpr>,
        right: Box<ProcessExpr>,
    },
    Parallel {
        left: Box<ProcessExpr>,
        right: Box<ProcessExpr>,
    },
    Communication {
        left: Box<ProcessExpr>,
        right: Box<ProcessExpr>,
        actions: Vec<CommAction>,
    },
    Hide {
        actions: Vec<String>,
        operand: Box<ProcessExpr>,
    },
    Rename {
        renames: Vec<(String, String)>,
        operand: Box<ProcessExpr>,
    },
    Allow {
        actions: Vec<String>,
        operand: Box<ProcessExpr>,
    },
    Block {
        actions: Vec<String>,
        operand: Box<ProcessExpr>,
    },
    Condition {
        condition: DataExpr,
        then: Box<ProcessExpr>,
        else_: Option<Box<ProcessExpr>>,
    },
}

/// Communication action
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct CommAction {
    pub inputs: Vec<String>,
    pub output: String,
    pub span: Span,
}

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

impl fmt::Display for UntypedProcessSpecification {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{:?}", self.data_specification)?;
        Ok(())
    }
}

impl fmt::Display for UntypedDataSpecification {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for decl in &self.sort_decls {
            writeln!(f, "{:?}", decl)?;
        }
        for decl in &self.cons_decls {
            writeln!(f, "{:?}", decl)?;
        }
        for decl in &self.map_decls {
            writeln!(f, "{:?}", decl)?;
        }
        for decl in &self.var_decls {
            writeln!(f, "{:?}", decl)?;
        }
        for decl in &self.eqn_decls {
            writeln!(f, "{:?}", decl)?;
        }
        Ok(())
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
