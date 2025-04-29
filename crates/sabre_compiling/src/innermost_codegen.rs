use std::collections::HashSet;
use std::fmt;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use indoc::indoc;
use mcrl3_sabre::RewriteSpecification;
use mcrl3_sabre::SetAutomaton;
use mcrl3_sabre::utilities::ExplicitPosition;
use mcrl3_utilities::MCRL3Error;

use crate::indenter::IndentFormatter;

/// Generates Rust code for term rewriting based on the provided specification.
///
/// Takes a rewrite specification and a source directory path, and generates the
/// necessary code for term rewriting using an automaton-based approach.
pub fn generate(spec: &RewriteSpecification, source_dir: &Path) -> Result<(), MCRL3Error> {
    let mut file = File::create(PathBuf::from(source_dir).join("lib.rs"))?;

    // Create an indented formatter for clean code generation
    let mut formatter = IndentFormatter::new(&mut file);

    // Generate the automata used for matching
    let apma = SetAutomaton::new(spec, |_| (), true);
    
    // Debug assertion to verify we have at least one state in the automaton
    debug_assert!(!apma.states().is_empty(), "Automaton must have at least one state");

    // Write imports and the main rewrite function
    writeln!(
        &mut formatter,
        indoc! {"use mcrl3_sabre_ffi::DataExpression;
        use mcrl3_sabre_ffi::DataExpressionRef;

        /// Generic rewrite function
        #[unsafe(no_mangle)]
        pub unsafe extern \"C\" fn rewrite(term: &DataExpressionRef<'_>) -> DataExpression {{
            rewrite_0(&term.copy())
        }}
        "}
    )?;

    // Introduce a match function for every state of the set automaton.
    let mut positions: HashSet<ExplicitPosition> = HashSet::new();

    for (index, state) in apma.states().iter().enumerate() {
        writeln!(
            &mut formatter,
            "fn rewrite_{}(t: &DataExpressionRef<'_>) -> DataExpression {{",
            index
        )?;
        
        // Use the IndentFormatter to properly indent the function body
        let _indent = formatter.indent();
        
        writeln!(
            &mut formatter,
            "let arg = get_position_{}(t);",
            UnderscoreFormatter(state.label())
        )?;
        writeln!(&mut formatter, "let symbol = arg.data_function_symbol();")?;

        positions.insert(state.label().clone());

        writeln!(&mut formatter, "match symbol.operation_id() {{")?;
        
        // Indent the match block
        let _match_indent = formatter.indent();

        for ((from, symbol), transition) in apma.transitions() {
            if *from == index {
                // Outgoing transition
                writeln!(&mut formatter, "{symbol} => {{")?;
                
                // Indent the case block
                let _case_indent = formatter.indent();

                // Continue on the outgoing transition.
                for (_announcement, _annotation) in transition.announcements() {
                    // Check for conditions and non linear patterns.
                    writeln!(
                        &mut formatter,
                        "t.protect()"
                    )?;
                }

                for (position, to) in transition.destinations() {
                    positions.insert(position.clone());

                    writeln!(
                        &mut formatter,
                        "rewrite_{to}(&get_position_{}(t))",
                        UnderscoreFormatter(position)
                    )?;
                }

                // The case block indent is automatically decreased when _case_indent goes out of scope
                writeln!(&mut formatter, "}}")?;
            }
        }

        // No match
        writeln!(&mut formatter, "_ => {{")?;
        
        // Indent the default case
        let _default_indent = formatter.indent();
        writeln!(&mut formatter, "t.protect()")?;
        
        // The default case indent is automatically decreased
        writeln!(&mut formatter, "}}")?;

        // The match indent is automatically decreased
        writeln!(&mut formatter, "}}")?;
        
        // The function indent is automatically decreased
        writeln!(&mut formatter, "}}")?;
        writeln!(&mut formatter)?;
    }

    // Introduce getters for all the positions that must be read from terms.
    for position in &positions {
        writeln!(
            &mut formatter,
            "fn get_position_{}<'a>(t: &DataExpressionRef<'a>) -> DataExpressionRef<'a> {{",
            UnderscoreFormatter(position)
        )?;
        
        // Indent the function body
        let _indent = formatter.indent();

        if position.is_empty() {
            writeln!(&mut formatter, "t.copy()")?;
        }
        else
        {
            write!(&mut formatter, "t")?;

            for index in &position.indices {
                write!(&mut formatter, ".arg({index})")?;
            }
            
            // Add newline after the chain of method calls
            writeln!(&mut formatter)?;
        }

        // The function indent is automatically decreased
        writeln!(&mut formatter, "}}")?;
        writeln!(&mut formatter)?;
    }

    // Ensure all data is written
    formatter.flush()?;

    // Post-condition assertion
    debug_assert!(!positions.is_empty(), "At least one position should be generated");

    Ok(())
}

struct UnderscoreFormatter<'a>(&'a ExplicitPosition);

impl fmt::Display for UnderscoreFormatter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.indices.is_empty() {
            write!(f, "epsilon")?;
        } else {
            let mut first = true;
            for p in &self.0.indices {
                if first {
                    write!(f, "{}", p)?;
                    first = false;
                } else {
                    write!(f, "_{}", p)?;
                }
            }
        }

        Ok(())
    }
}
