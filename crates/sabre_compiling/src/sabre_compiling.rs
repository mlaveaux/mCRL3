use std::path::Path;
use std::path::PathBuf;

use libloading::Library;
use libloading::Symbol;
use log::info;
use mcrl3_aterm::ATermRef;
use mcrl3_aterm::Term;
use mcrl3_utilities::MCRL3Error;
use tempfile::tempdir;
use toml::Table;

use mcrl3_data::DataExpression;
use mcrl3_sabre::RewriteEngine;
use mcrl3_sabre::RewriteSpecification;
use mcrl3_sabre_ffi::DataExpression as DataExpressionFFI;
use mcrl3_sabre_ffi::DataExpressionRef as DataExpressionRefFFI;

use crate::generate;
use crate::library::RuntimeLibrary;

pub struct SabreCompilingRewriter {
    library: Library,
    //rewrite_func: Symbol<unsafe extern fn() -> u32>,
}

impl RewriteEngine for SabreCompilingRewriter {
    fn rewrite(&mut self, term: &DataExpression) -> DataExpression {
        // TODO: This ought to be stored somewhere for repeated calls.
        unsafe {
            let func: Symbol<extern "C" fn(&DataExpressionRefFFI) -> DataExpressionFFI> =
                self.library.get(b"rewrite").unwrap();

            let result = func(&DataExpressionRefFFI::from_index(term.shared()));
            ATermRef::from_index(result.index()).protect().into()
        }
    }
}

impl SabreCompilingRewriter {
    /// Creates a new compiling rewriter for the given specifications.
    ///
    /// - use_local_workspace: Use the development version of the toolset instead of referring to the github one.
    /// - use_local_tmp: Use a relative 'tmp' directory instead of using the system directory. Mostly used for debugging purposes.
    ///
    /// - [`RewriteEngine`]
    pub fn new(
        spec: &RewriteSpecification,
        use_local_workspace: bool,
        use_local_tmp: bool,
    ) -> Result<SabreCompilingRewriter, MCRL3Error> {
        let system_tmp_dir = tempdir()?;
        let temp_dir = if use_local_tmp {
            Path::new("./tmp")
        } else {
            system_tmp_dir.path()
        };

        let mut dependencies = vec![];

        if use_local_workspace {
            let compilation_toml = include_str!("../../../target/Compilation.toml").parse::<Table>()?;
            let path = compilation_toml
                .get("sabrec")
                .ok_or("Missing [sabre] section)")?
                .get("path")
                .ok_or("Missing path entry")?
                .as_str()
                .ok_or("Not a string")?;

            info!("Using local dependency {}", path);
            dependencies.push(format!(
                "mcrl3_sabre-ffi = {{ path = '{}' }}",
                PathBuf::from(path)
                    .join("../../crates/sabre_compiling/sabre_ffi")
                    .to_string_lossy()
            ));
        } else {
            info!("Using git dependency https://github.com/mlaveaux/mCRL3.git");
            dependencies.push("mcrl3_sabre-ffi = { git = 'https://github.com/mlaveaux/mCRL3.git' }".to_string());
        }

        let mut compilation_crate = RuntimeLibrary::new(temp_dir, dependencies)?;

        // Write the output source file(s).
        generate(spec, compilation_crate.source_dir())?;

        let library = compilation_crate.compile()?;
        Ok(SabreCompilingRewriter { library })
    }
}

#[cfg(test)]
mod tests {
    use ahash::AHashSet;
    use mcrl3_data::to_untyped_data_expression;
    use mcrl3_rec_tests::load_rec_from_strings;
    use mcrl3_sabre::RewriteEngine;

    use super::SabreCompilingRewriter;

    #[test]
    fn test_compilation() {
        let (spec, terms) = load_rec_from_strings(&[
            include_str!("../../../examples/REC/rec/factorial6.rec"),
            include_str!("../../../examples/REC/rec/factorial.rec"),
        ])
        .unwrap();

        let spec = spec.to_rewrite_spec();

        let mut rewriter = SabreCompilingRewriter::new(&spec, true, true).unwrap();

        for t in terms {
            let data_term = to_untyped_data_expression(&t, &AHashSet::new());
            assert_eq!(
                rewriter.rewrite(&data_term),
                data_term,
                "The rewritten result does not match the expected result"
            );
        }
    }
}
