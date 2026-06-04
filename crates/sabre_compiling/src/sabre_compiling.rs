use std::ffi::c_void;
use std::path::Path;
use std::path::PathBuf;

use libloading::Library;
use libloading::Symbol;
use log::info;
use tempfile::tempdir;
use toml::Table;

use merc_data::DataExpression;
use merc_sabre::RewriteEngine;
use merc_sabre::RewriteSpecification;
use merc_sabre_ffi::DataExpressionFFI;
use merc_sabre_ffi::DataExpressionRefFFI;
use merc_sabre_ffi::SabreRewriteVTable;
use merc_sabre_ffi::data_expression_ref_from_term;
use merc_sabre_ffi::into_data_expression;
use merc_sabre_ffi::rewrite_vtable;
use merc_utilities::MercError;

use crate::generate;
use crate::library::RuntimeLibrary;

pub struct SabreCompilingRewriter {
    library: Library,
    /// Whether the host vtable has already been installed into `library`.
    initialised: bool,
}

impl RewriteEngine for SabreCompilingRewriter {
    fn rewrite(&mut self, term: &DataExpression) -> DataExpression {
        unsafe {
            // Install the host vtable into the loaded library exactly once. All
            // term-pool access in the library is routed back through it, so the
            // library never touches its own (duplicated) term pool.
            if !self.initialised {
                let initialise: Symbol<extern "C-unwind" fn(*mut c_void)> = self.library.get(b"initialise").unwrap();

                let vtable: SabreRewriteVTable = rewrite_vtable();
                initialise(std::ptr::addr_of!(vtable) as *mut c_void);
                self.initialised = true;
            }

            let func: Symbol<extern "C-unwind" fn(&DataExpressionRefFFI) -> DataExpressionFFI> =
                self.library.get(b"rewrite").unwrap();

            let result = func(&data_expression_ref_from_term(term));
            into_data_expression(result)
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
    ) -> Result<SabreCompilingRewriter, MercError> {
        let system_tmp_dir = tempdir()?;
        let temp_dir = if use_local_tmp {
            Path::new("./tmp")
        } else {
            system_tmp_dir.path()
        };

        let mut dependencies = vec![];

        if use_local_workspace {
            let compilation_toml = include_str!(concat!(env!("OUT_DIR"), "/Compilation.toml")).parse::<Table>()?;
            let path = compilation_toml
                .get("sabrec")
                .ok_or("Missing [sabre] section)")?
                .get("path")
                .ok_or("Missing path entry")?
                .as_str()
                .ok_or("Not a string")?;

            info!("Using local dependency {path}");
            dependencies.push(format!(
                "merc_sabre-ffi = {{ path = '{}' }}",
                PathBuf::from(path)
                    .join("../../crates/sabre_compiling/sabre_ffi")
                    .to_string_lossy()
            ));
        } else {
            info!("Using git dependency https://github.com/mlaveaux/merc.git");
            dependencies.push("merc_sabre-ffi = { git = 'https://github.com/mlaveaux/merc.git' }".to_string());
        }

        let mut compilation_crate = RuntimeLibrary::new(temp_dir, dependencies)?;

        // Write the output source file(s).
        generate(spec, compilation_crate.source_dir())?;

        let library = compilation_crate.compile()?;
        Ok(SabreCompilingRewriter {
            library,
            initialised: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use test_log::test;

    use merc_data::to_untyped_data_expression;
    use merc_rec_tests::load_rec_from_strings;
    use merc_sabre::RewriteEngine;

    use super::SabreCompilingRewriter;

    #[test]
    #[cfg_attr(miri, ignore)] // Miri does not support FFI.
    fn test_sabre_compiling_example() {
        let (spec, terms) = load_rec_from_strings(&[
            include_str!("../../../examples/REC/rec/factorial5.rec"),
            include_str!("../../../examples/REC/rec/factorial.rec"),
        ])
        .unwrap();

        let spec = spec.to_rewrite_spec();
        let mut rewriter = SabreCompilingRewriter::new(&spec, true, true).unwrap();

        for t in terms {
            let data_term = to_untyped_data_expression(t, None);
            let rewritten_term = rewriter.rewrite(&data_term);

            println!("Original term: {data_term}");
            println!("Rewritten term: {rewritten_term}");

            assert_eq!(
                rewritten_term.to_string().chars().filter(|c| *c == 's').count(),
                120, // 5! = 120.
                "The rewritten result does not match the expected result"
            );
        }
    }
}
