use std::collections::HashSet;
use std::fmt;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use indoc::indoc;

use itertools::Itertools;
use merc_aterm::Term;
use merc_sabre::AnnouncementInnermost;
use merc_sabre::RewriteSpecification;
use merc_sabre::SetAutomaton;
use merc_sabre::matching::conditions::EMACondition;
use merc_sabre::matching::nonlinear::EquivalenceClass;
use merc_sabre::utilities::Config;
use merc_sabre::utilities::DataPosition;
use merc_sabre::utilities::TermStack;
use merc_utilities::MercError;

use crate::indenter::IndentFormatter;

/// Generates Rust code for term rewriting based on the provided specification.
///
/// Takes a rewrite specification and a source directory path, and generates the
/// necessary code for term rewriting using an automaton-based approach.
pub fn generate(spec: &RewriteSpecification, source_dir: &Path) -> Result<(), MercError> {
    let mut file = File::create(PathBuf::from(source_dir).join("lib.rs"))?;

    // Create an indented formatter for clean code generation
    let mut formatter = IndentFormatter::new(&mut file);

    // Generate the automata used for matching
    let apma = SetAutomaton::new(spec, AnnouncementInnermost::new, true);

    // Debug assertion to verify we have at least one state in the automaton
    debug_assert!(!apma.states().is_empty(), "Automaton must have at least one state");

    // Write imports and the main rewrite function
    writeln!(
        &mut formatter,
        indoc! {"#![allow(unused_variables)]
        #![allow(improper_ctypes_definitions)]

        use std::ffi::c_void;
        
        use merc_sabre_ffi::initialize_thread_local_term_pool;
        use merc_sabre_ffi::SymbolRefFFI;
        use merc_sabre_ffi::DataExpressionFFI;
        use merc_sabre_ffi::DataExpressionRefFFI;

        /// The initialisation function used to pass the GLOBAL_TERM_POOL to the shared library.
        #[unsafe(no_mangle)]
        pub unsafe extern \"C\" fn initialise(global_term_pool: *mut c_void) {{
            initialize_thread_local_term_pool(global_term_pool);
        }}

        /// Generic rewrite function
        #[unsafe(no_mangle)]
        pub unsafe extern \"C\" fn rewrite(term: &DataExpressionRefFFI<'_>) -> DataExpressionFFI {{
            rewrite_0(&term.copy())
        }}
        "}
    )?;

    // Keep track of all positions that need to be read from terms, to generate getters for them later.
    let mut positions: HashSet<DataPosition> = HashSet::new();

    // Introduce a match function for every state of the set automaton.
    for (index, state) in apma.states().iter().enumerate() {
        writeln!(&mut formatter, "// Position {}", state.label())?;

        for goal in state.match_goals() {
            writeln!(&mut formatter, "// Goal {goal:?}")?;
        }

        writeln!(
            &mut formatter,
            "fn rewrite_{index}(t: &DataExpressionRefFFI<'_>) -> DataExpressionFFI {{"
        )?;

        // Use the IndentFormatter to properly indent the function body
        let indent = formatter.indent();

        writeln!(
            &mut formatter,
            "let arg = get_data_position_{}(t);",
            UnderscoreFormatter(state.label())
        )?;
        writeln!(&mut formatter, "let symbol = arg.data_function_symbol();")?;

        positions.insert(state.label().clone());

        writeln!(&mut formatter, "match symbol.operation_id() {{")?;

        // Indent the match block
        let match_indent = formatter.indent();

        for ((from, symbol), transition) in apma.transitions() {
            // Consider only transitions that match the current state index
            if *from == index {
                writeln!(&mut formatter, "{symbol} => {{")?;

                // Indent the case block
                let case_indent = formatter.indent();
                writeln!(&mut formatter, "// Symbol {}", transition.symbol)?;

                // Continue on the outgoing transition.
                for (announcement, _annotation) in &transition.announcements {
                    // Check for conditions and non linear patterns.
                    writeln!(&mut formatter, "// Announcement {announcement:?}")?;

                    writeln!(
                        &mut formatter,
                        "if check_equivalence_classes_{index}_{symbol}(t) && check_condition_{index}_{symbol}(t) {{",
                    )?;

                    let condition_indent = formatter.indent();
                    writeln!(&mut formatter, "return rewrite_term_stack_{index}_{symbol}(t)")?;
                    drop(condition_indent);

                    writeln!(&mut formatter, "}}")?;
                }

                if transition.destinations.is_empty() {
                    writeln!(&mut formatter, "t.protect()")?;
                }

                for (position, to) in &transition.destinations {
                    positions.insert(position.clone());

                    writeln!(&mut formatter, "rewrite_{to}(&t)",)?;
                }

                drop(case_indent);
                writeln!(&mut formatter, "}}")?;
            }
        }

        // No match
        writeln!(&mut formatter, "_ => {{")?;

        // Indent the default case
        {
            let _default_indent = formatter.indent();
            writeln!(&mut formatter, "t.protect()")?;
        }

        writeln!(&mut formatter, "}}")?;

        drop(match_indent);
        writeln!(&mut formatter, "}}")?;

        drop(indent);
        writeln!(&mut formatter, "}}")?;
        writeln!(&mut formatter)?;
    }

    writeln!(formatter, "// term stack rewrite functions")?;
    writeln!(formatter)?;
    for ((from, symbol), transition) in apma.transitions() {
        for (_announcement, annotation) in &transition.announcements {
            generate_rewrite_term_stack(&mut formatter, *from, *symbol, &mut positions, &annotation.rhs_stack)?;
        }
    }

    writeln!(formatter, "// check condition functions")?;
    writeln!(formatter)?;
    for ((from, symbol), transition) in apma.transitions() {
        for (_announcement, annotation) in &transition.announcements {
            generate_check_condition(&mut formatter, *from, *symbol, &annotation.conditions)?;
        }
    }

    writeln!(formatter, "// equivalence classes check")?;
    writeln!(formatter)?;
    for ((from, symbol), transition) in apma.transitions() {
        for (_announcement, annotation) in &transition.announcements {
            generate_check_equivalence_classes(&mut formatter, *from, *symbol, &annotation.equivalence_classes)?;
        }
    }

    // Generate position getters
    writeln!(formatter, "// position getters")?;
    writeln!(formatter)?;
    generate_position_getters(&mut formatter, &positions)?;

    // Ensure all data is written
    formatter.flush()?;

    // Post-condition assertion
    debug_assert!(!positions.is_empty(), "At least one position should be generated");

    Ok(())
}

fn generate_check_condition(
    formatter: &mut IndentFormatter<File>,
    index: usize,
    symbol: usize,
    conditions: &[EMACondition],
) -> Result<(), MercError> {
    writeln!(formatter, "/// Checking condition {:?}", conditions)?;

    writeln!(
        formatter,
        "fn check_condition_{index}_{symbol}(t: &DataExpressionRefFFI<'_>) -> bool {{"
    )?;

    let condition_indent = formatter.indent();
    for condition in conditions {
        writeln!(formatter, "// Condition {condition:?}")?;
        writeln!(formatter, "let left = rewrite_term_stack(t)")?;
        writeln!(formatter, "let right = rewrite_term_stack(t)")?;

        writeln!(formatter, "if left == right {{")?;

        let condition_body_indent = formatter.indent();
        writeln!(formatter, "return true")?;
        drop(condition_body_indent);

        writeln!(formatter, "}}")?;
    }

    writeln!(formatter, "true")?;
    drop(condition_indent);

    writeln!(formatter, "}}")?;
    Ok(())
}

/// Generates `rewrite_term_stack` functions for the given term stack, which
/// perform the actual rewriting based on the innermost strategy.
fn generate_rewrite_term_stack(
    formatter: &mut IndentFormatter<File>,
    index: usize,
    symbol: usize,
    positions: &mut HashSet<DataPosition>,
    term_stack: &TermStack,
) -> Result<(), MercError> {
    writeln!(formatter, "/// Rewriting {:?}", term_stack)?;
    writeln!(
        formatter,
        "fn rewrite_term_stack_{index}_{symbol}(t: &DataExpressionRefFFI<'_>) -> DataExpressionFFI {{"
    )?;

    let indent = formatter.indent();

    // Get the terms at the positions that need to be read.
    for (position, stack_index) in &term_stack.variables {
        positions.insert(position.clone());
        writeln!(
            formatter,
            "let var_{stack_index} = get_data_position_{}(t);",
            UnderscoreFormatter(position)
        )?;
    }

    let read = term_stack.innermost_stack.read();
    for config in read.iter().rev() {
        match config {
            Config::Construct(symbol, arity, stack_index) => {
                if *arity > 0 {
                    writeln!(
                        formatter,
                        "let var_{stack_index} = DataExpressionFFI::create(unsafe {{ SymbolRefFFI::from_ptr({:?}) }}, &[{}]);",
                        symbol.shared().ptr().as_ptr() as *mut () as usize,
                        (0..*arity)
                            .map(|i| format!("var_{}.copy()", stack_index + i + 1))
                            .format(", ")
                    )?;
                } else {
                    writeln!(
                        formatter,
                        "let var_{stack_index} = DataExpressionFFI::constant(unsafe {{ SymbolRefFFI::from_ptr({:?}) }});",
                        symbol.shared().ptr().as_ptr() as *mut () as usize,
                    )?;
                }
            }
            Config::Term(data_expression_ref, index) => {
                writeln!(
                    formatter,
                    "let var_{index} = unsafe {{ DataExpressionFFI::from_ptr({:?}) }};",
                    data_expression_ref.shared().ptr().as_ptr() as *mut () as usize
                )?;
            }
            Config::Rewrite(_) | Config::Return() => unreachable!("The term stack never contains these configurations"),
        }
    }

    // Return the last variable, which contains the result of the rewrite.
    if let Some(last) = term_stack.innermost_stack.read().iter().rev().find_map(|config| {
        if let Config::Construct(_, _, stack_index) = config {
            Some(stack_index)
        } else {
            None
        }
    }) {
        writeln!(formatter, "var_{last}.protect()")?;
    } else {
        writeln!(formatter, "t.protect()")?;
    }

    drop(indent);
    writeln!(formatter, "}}")?;
    writeln!(formatter)?;

    Ok(())
}

/// Generates getter functions for all positions that must be read from terms.
fn generate_position_getters(
    formatter: &mut IndentFormatter<File>,
    positions: &HashSet<DataPosition>,
) -> Result<(), MercError> {
    for position in positions {
        writeln!(formatter, "/// Get position {:?} from term", position)?;
        writeln!(
            formatter,
            "fn get_data_position_{}<'a>(t: &DataExpressionRefFFI<'a>) -> DataExpressionRefFFI<'a> {{",
            UnderscoreFormatter(position)
        )?;

        // Indent the function body
        let indent = formatter.indent();

        if position.is_empty() {
            writeln!(formatter, "t.copy()")?;
        } else {
            write!(formatter, "t")?;

            for index in position.indices().iter() {
                write!(formatter, ".data_arg({})", index - 1)?; // Note that positions are 1 indexed.
            }

            // Add newline after the chain of method calls
            writeln!(formatter)?;
        }

        // The function indent is automatically decreased
        drop(indent);
        writeln!(formatter, "}}")?;
        writeln!(formatter)?;
    }

    Ok(())
}

/// Generates getter functions for all positions that must be read from terms.
fn generate_check_equivalence_classes(
    formatter: &mut IndentFormatter<File>,
    index: usize,
    symbol: usize,
    equivalence_classes: &Vec<EquivalenceClass>,
) -> Result<(), MercError> {
    writeln!(formatter, "/// Check equivalence classes")?;
    writeln!(
        formatter,
        "fn check_equivalence_classes_{index}_{symbol}<'a>(t: &DataExpressionRefFFI<'a>) -> bool {{",
    )?;

    // Indent the function body
    let indent = formatter.indent();
    if let Some(first) = equivalence_classes.first() {
        equivalence_classes.iter().skip(1).for_each(|class| {
            writeln!(formatter, "&& ({} == {})", class, first).unwrap();
        });
    } else {
        writeln!(formatter, "true")?;
    }

    // The function indent is automatically decreased
    drop(indent);

    writeln!(formatter, "}}")?;
    writeln!(formatter)?;
    Ok(())
}

struct UnderscoreFormatter<'a>(&'a DataPosition);

impl fmt::Display for UnderscoreFormatter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            write!(f, "epsilon")?;
        } else {
            let mut first = true;
            for p in self.0.indices().iter() {
                if first {
                    write!(f, "{p}")?;
                    first = false;
                } else {
                    write!(f, "_{p}")?;
                }
            }
        }

        Ok(())
    }
}
