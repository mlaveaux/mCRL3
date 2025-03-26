use std::error::Error;
use std::path::Path;
use std::path::PathBuf;

use libloading::Library;
use libloading::Symbol;
use log::info;
use temp_dir::TempDir;
use toml::Table;

use mcrl3_data::DataExpression;
use mcrl3_sabre::RewriteEngine;
use mcrl3_sabre::RewriteSpecification;

use crate::generate;
use crate::library::RuntimeLibrary;

pub struct SabreCompilingRewriter {
    library: Library,
    //rewrite_func: Symbol<unsafe extern fn() -> u32>,
}

impl RewriteEngine for SabreCompilingRewriter {
    fn rewrite(&mut self, term: DataExpression) -> DataExpression {
        // TODO: This ought to be stored somewhere for repeated calls.
        unsafe {
            let func: Symbol<extern "C" fn(&DataExpression) -> DataExpression> = self.library.get(b"rewrite").unwrap();

            let result = func(&term);
            std::mem::forget(result);

            term
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
    ) -> Result<SabreCompilingRewriter, Box<dyn Error>> {
        let system_tmp_dir = TempDir::new()?;
        let temp_dir = if use_local_tmp {
            &Path::new("./tmp")
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
                "mcrl2 = {{ path = '{}' }}",
                PathBuf::from(path).join("../mcrl2").to_string_lossy()
            ));
        } else {
            info!("Using git dependency https://github.com/mlaveaux/mCRL3.git");
            dependencies.push("sabre-ffi = { git = 'https://github.com/mlaveaux/mCRL3.git' }".to_string());
        }

        let mut compilation_crate = RuntimeLibrary::new(temp_dir, dependencies)?;

        // Write the output source file(s).
        generate(spec, compilation_crate.source_dir())?;

        let library = compilation_crate.compile()?;
        Ok(SabreCompilingRewriter { library })
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     use mcrl3_data::DataSpecification;
//     use test_log::test;

//     #[test]
//     fn test_compilation() {

//         let spec = DataSpecification::new("
//             map f: Bool # Bool -> Bool;
//             eqn f(true, false) = false;
//         ").unwrap();

//         let t = spec.parse("f(true, false)").unwrap();

//         let spec = RewriteSpecification::from(spec);
//         let tp = Rc::new(RefCell::new(TermPool::new()));

//         let mut rewriter = SabreCompilingRewriter::new(tp, &spec, true, true).unwrap();

//         assert_eq!(rewriter.rewrite(t.clone()), t, "The rewritten result does not match the expected result");
//     }
// }
