use std::fmt;

use ahash::AHashSet;
use log::trace;
use mcrl3_utilities::MCRL3Error;

use crate::ATerm;
use crate::Symbol;
use crate::THREAD_TERM_POOL;
use crate::Term;
use crate::ThreadTermPool;

/// This can be used to construct a term from a given input of (inductive) type I,
/// without using the system stack, i.e. recursion. See evaluate.
#[derive(Default)]
pub struct TermBuilder<I, C> {
    // The stack of terms
    terms: Vec<Option<ATerm>>,
    configs: Vec<Config<I, C>>,
}

/// Applies the given function to every subterm of the given term using the [TermBuilder].
///     function(subterm) returns:
///         None   , in which case subterm is kept and it is recursed into its argments.
///         Some(x), in which case subterm is replaced by x.
pub fn apply<F>(tp: &ThreadTermPool, t: &ATerm, function: &F) -> ATerm
where
    F: Fn(&ThreadTermPool, &ATerm) -> Option<ATerm>,
{
    let mut builder = TermBuilder::<ATerm, Symbol>::new();

    builder
        .evaluate(
            tp,
            t.clone(),
            |tp, args, t| match function(tp, &t) {
                Some(result) => Ok(Yield::Term(result)),
                None => {
                    for arg in t.arguments() {
                        args.push(arg.protect());
                    }

                    Ok(Yield::Construct(t.get_head_symbol().protect()))
                }
            },
            |tp, symbol, args| Ok(tp.create_term_iter(&symbol, args)),
        )
        .unwrap()
}

impl<I: fmt::Debug, C: fmt::Debug> TermBuilder<I, C> {
    pub fn new() -> TermBuilder<I, C> {
        TermBuilder {
            terms: vec![],
            configs: vec![],
        }
    }

    /// This can be used to construct a term from a given input of (inductive)
    /// type I, without using the system stack, i.e. recursion.
    ///
    /// The `transformer` function is applied to every instance I, which can put
    /// more generate more inputs using a so-called argument stack and some
    /// instance C that is used to construct the result term. Alternatively, it
    /// yields a result term directly.
    ///
    /// The `construct` function takes an instance C and the arguments pushed to
    /// stack where the transformer was applied for every input pushed onto the
    /// stack previously.
    ///
    /// # Example
    ///
    /// A simple example could be to transform a term into another term using a
    /// function `f : ATerm -> Option<ATerm>`. Then `I` will be ATerm since that is
    /// the input, and `C` will be the Symbol from which we can construct the
    /// recursive term.
    ///
    /// `transformer` takes the input and applies f(input). Then either we
    /// return Yield(x) when f returns some term, or Construct(head(input)) with
    /// the arguments of the input term pushed to stack.
    ///
    /// `construct` simply constructs the term from the symbol and the arguments
    /// on the stack.
    ///
    /// However, it can also be that I is some syntax tree from which we want to
    /// construct a term.
    pub fn evaluate<F, G>(
        &mut self,
        tp: &ThreadTermPool,
        input: I,
        transformer: F,
        construct: G,
    ) -> Result<ATerm, MCRL3Error>
    where
        F: Fn(&ThreadTermPool, &mut ArgStack<I, C>, I) -> Result<Yield<C>, MCRL3Error>,
        // We need impl<Iterator<Item=&ATerm>> here, but that is not possible.
        G: Fn(&ThreadTermPool, C, std::iter::Flatten<std::slice::Iter<Option<ATerm>>>) -> Result<ATerm, MCRL3Error>,
    {
        trace!("Transforming {:?}", input);
        self.terms.push(None);
        self.configs.push(Config::Apply(input, 0));

        while let Some(config) = self.configs.pop() {
            match config {
                Config::Apply(input, result) => {
                    // Applies the given function to this input, and obtain a number of symbol and arguments.
                    let top_of_stack = self.configs.len();
                    let mut args = ArgStack::new(&mut self.terms, &mut self.configs);

                    match transformer(tp, &mut args, input)? {
                        Yield::Construct(input) => {
                            // This occurs before the other constructs.
                            let arity = args.len();
                            self.configs.reserve(1);
                            self.configs
                                .insert(top_of_stack, Config::Construct(input, arity, result));
                        }
                        Yield::Term(term) => {
                            self.terms[result] = Some(term);
                        }
                    }
                }
                Config::Construct(input, arity, result) => {
                    let arguments = self.terms[self.terms.len() - arity..].iter().flatten();

                    self.terms[result] = Some(construct(tp, input, arguments)?);

                    // Remove elements from the stack.
                    self.terms.drain(self.terms.len() - arity..);
                }
            }

            trace!("{:?}", self);
        }

        debug_assert!(self.terms.len() == 1, "Expect exactly one term on the result stack");

        Ok(self
            .terms
            .pop()
            .expect("There should be at last one result")
            .expect("The result should be Some"))
    }
}

enum Config<I, C> {
    Apply(I, usize),
    Construct(C, usize, usize),
}

pub enum Yield<C> {
    Term(ATerm),  // Yield this term as is.
    Construct(C), // Yield f(args) for every arg push to the argument stack, with the function applied to it.
}

/// This struct defines a local argument stack on the global stack.
pub struct ArgStack<'a, I, C> {
    terms: &'a mut Vec<Option<ATerm>>,
    configs: &'a mut Vec<Config<I, C>>,
    top_of_stack: usize,
}

impl<'a, I, C> ArgStack<'a, I, C> {
    fn new(terms: &'a mut Vec<Option<ATerm>>, configs: &'a mut Vec<Config<I, C>>) -> ArgStack<'a, I, C> {
        let top_of_stack = terms.len();
        ArgStack {
            terms,
            configs,
            top_of_stack,
        }
    }

    /// Returns the amount of arguments added.
    fn len(&self) -> usize {
        self.terms.len() - self.top_of_stack
    }

    /// Adds the term to the argument stack, will construct construct(C, args...) with the transformer applied to arguments.
    pub fn push(&mut self, input: I) {
        self.configs.push(Config::Apply(input, self.terms.len()));
        self.terms.push(None);
    }
}

impl<I: fmt::Debug, C: fmt::Debug> fmt::Debug for TermBuilder<I, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Terms: [")?;
        for (i, term) in self.terms.iter().enumerate() {
            writeln!(f, "{}\t{:?}", i, term)?;
        }
        writeln!(f, "]")?;

        writeln!(f, "Configs: [")?;
        for config in &self.configs {
            writeln!(f, "\t{:?}", config)?;
        }
        write!(f, "]")
    }
}

impl<I: fmt::Debug, C: fmt::Debug> fmt::Debug for Config<I, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Config::Apply(x, result) => write!(f, "Apply({:?}, {})", x, result),
            Config::Construct(symbol, arity, result) => {
                write!(f, "Construct({:?}, {}, {})", symbol, arity, result)
            }
        }
    }
}

/// Create a random term consisting of the given symbol and constants. Performs
/// iterations number of constructions, and uses chance_duplicates to choose the
/// amount of subterms that are duplicated.
pub fn random_term(
    rng: &mut impl rand::Rng,
    symbols: &[(String, usize)],
    constants: &[String],
    iterations: usize,
) -> ATerm {
    use rand::prelude::IteratorRandom;

    debug_assert!(!constants.is_empty(), "We need constants to be able to create a term");

    let mut subterms = THREAD_TERM_POOL.with_borrow(|tp| {
        AHashSet::<ATerm>::from_iter(constants.iter().map(|name| {
            let symbol = tp.create_symbol(name, 0);
            let a: &[ATerm] = &[];
            tp.create_term(&symbol, a)
        }))
    });

    let mut result = None;
    for _ in 0..iterations {
        let (symbol, arity) = symbols.iter().choose(rng).unwrap();

        let mut arguments = vec![];
        for _ in 0..*arity {
            arguments.push(subterms.iter().choose(rng).unwrap().clone());
        }

        let symbol = Symbol::new(symbol, *arity);
        let term = ATerm::with_args(&symbol, &arguments);

        // Make this term available as another subterm that can be used.
        subterms.insert(term.clone());

        result = Some(term);
    }

    result.expect("At least one iteration was performed")
}
