use crate::linter::clippy::Clippy;
use crate::linter::Lint;
use std::env;
use structopt::StructOpt;

mod error;
mod git;
mod linter;
mod project;
mod scout;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "cargo-scout",
    author,
    about = "Leave the codebase better than when you found it."
)]
struct Options {
    #[structopt(short = "v", long = "verbose")]
    /// Set the verbosity level
    verbose: bool,
    #[structopt(long = "no-default-features")]
    /// Pass the no default features flag to clippy
    no_default_features: bool,
    #[structopt(long = "all-features")]
    /// Pass the all features flag to clippy
    all_features: bool,
    #[structopt(
        short = "b",
        long = "branch",
        value_name = "branch",
        default_value = "master"
    )]
    /// Set the target branch
    branch: String,

    #[structopt(short = "t", long = "cargo-toml", default_value = "./Cargo.toml")]
    /// Pass the path of the `Cargo.toml` file
    cargo_toml: String,
    #[structopt(short = "w", long = "without-error")]
    /// Set to display the warnings without actually returning an error
    without_error: bool,
}

fn display_warnings(warnings: &[Lint]) {
    for w in warnings {
        if let Some(m) = &w.message {
            for l in m.rendered.split('\n') {
                println!("{}", l);
            }
        }
    }
    println!("Clippy::pedantic found {} warnings", warnings.len());
}

fn main() -> Result<(), error::Error> {
    let opts = Options::from_args();

    println!("Getting diff against target {}", opts.branch);
    let diff_sections = git::get_sections(env::current_dir()?, &opts.branch)?;

    println!("Checking Cargo manifest");
    let mut project = project::cargo::Project::from_manifest_path(opts.cargo_toml)?;
    project.set_all_features(opts.all_features);
    project.set_no_default_features(opts.no_default_features);

    let mut clippy_linter = Clippy::new();
    clippy_linter
        .set_verbose(opts.verbose)
        .set_no_default_features(opts.no_default_features)
        .set_all_features(opts.all_features);

    let mut scout_builder = scout::Builder::new();
    scout_builder
        .set_project_config(project)
        .set_linter(clippy_linter);
    println!("Running clippy");
    let scout = scout_builder.build()?;

    let relevant_lints = scout.run_for_diff(&diff_sections)?;
    return_warnings(&relevant_lints, opts.without_error)
}

fn return_warnings(lints: &[linter::Lint], without_error: bool) -> Result<(), error::Error> {
    if lints.is_empty() {
        println!("No warnings raised by clippy::pedantic in your diff, you're good to go!");
        Ok(())
    } else {
        display_warnings(&lints);
        if without_error {
            Ok(())
        } else {
            Err(error::Error::NotClean)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::linter::Lint;
    use crate::return_warnings;
    #[test]
    fn test_return_status_with_lints() {
        let lints = vec![Lint {
            package_id: "cargo-scout".to_string(),
            src_path: None,
            message: None,
        }];

        assert!(return_warnings(&lints, true).is_ok());
        assert!(return_warnings(&lints, false).is_err());
    }

    #[test]
    fn test_return_status_without_existing_lints() {
        let lints: Vec<Lint> = Vec::new();

        assert!(return_warnings(&lints, true).is_ok());
        assert!(return_warnings(&lints, false).is_ok());
    }
}
