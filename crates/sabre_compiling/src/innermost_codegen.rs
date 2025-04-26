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

pub fn generate(spec: &RewriteSpecification, source_dir: &Path) -> Result<(), MCRL3Error> {
    let mut file = File::create(PathBuf::from(source_dir).join("lib.rs"))?;

    // Generate the automata used for matching
    let apma = SetAutomaton::new(spec, |_| (), true);

    writeln!(
        &mut file,
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
            &mut file,
            "fn rewrite_{}(t: &DataExpressionRef<'_>) -> DataExpression {{",
            index
        )?;
        writeln!(
            &mut file,
            "\tlet arg = get_position_{}(t);",
            UnderscoreFormatter(state.label())
        )?;
        writeln!(&mut file, "\tlet symbol = arg.data_function_symbol();")?;

        positions.insert(state.label().clone());

        writeln!(&mut file, "\tmatch symbol.operation_id() {{")?;

        for ((from, symbol), transition) in apma.transitions() {
            if *from == index {
                // Outgoing transition
                writeln!(&mut file, "\t\t{symbol} => {{")?;

                // Continue on the outgoing transition.
                for (announcement, _annotation) in transition.announcements() {
                    // Check for conditions and non linear patterns.
                    writeln!(
                        &mut file,
                        "\t\t\tt.protect()"
                    )?;
                }

                for (position, to) in transition.destinations() {
                    positions.insert(position.clone());

                    writeln!(
                        &mut file,
                        "\t\t\trewrite_{to}(&get_position_{}(t))",
                        UnderscoreFormatter(position)
                    )?;
                }

                writeln!(&mut file, "\t\t}}")?;
            }
        }

        // No match
        writeln!(
            &mut file,
            indoc! {
            "\t\t_ => {{
            \t\t   t.protect()
            \t\t}}"}
        )?;

        writeln!(&mut file, "\t}}")?;
        writeln!(&mut file, "}}")?;
        writeln!(&mut file)?;
    }

    // Introduce getters for all the positions that must be read from terms.
    for position in &positions {
        writeln!(
            &mut file,
            "fn get_position_{}<'a>(t: &DataExpressionRef<'a>) -> DataExpressionRef<'a> {{",
            UnderscoreFormatter(position)
        )?;

        if position.is_empty() {
            writeln!(&mut file, "\tt.copy()")?;
        }
        else
        {
            writeln!(&mut file, "\tt")?;

            for index in &position.indices {
                write!(&mut file, "\t.arg({index})")?;
            }
        }

        writeln!(&mut file, "}}")?;
        writeln!(&mut file)?;
    }

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
