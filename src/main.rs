use structopt::StructOpt;

mod cargo;
mod clippy;
mod error;
mod git;
mod intersections;

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
    #[structopt(short = "W", long = "without-error")]
    /// Set to display the warnings without actually returning an error
    without_error: bool,
}

fn display_warnings(warnings: &[clippy::Lint]) {
    for w in warnings {
        if let Some(m) = &w.message {
            for l in m.rendered.split('\n') {
                println!("{}", l);
            }
        }
    }
}

fn main() -> Result<(), error::Error> {
    let opts = Options::from_args();

    println!("Getting diff against target {}", opts.branch);
    let diff_sections = git::Parser::new()
        .set_verbose(opts.verbose)
        .get_sections(&opts.branch)?;
    println!("Checking Cargo manifest");
    let manifest = cargo::Parser::from_manifest_path(opts.cargo_toml)?;
    if manifest.is_workspace() {
        println!("Running in workspace, please note feature flags are not supported yet.");
    }
    println!("Running clippy");
    let clippy_lints = clippy::Linter::new()
        .set_verbose(opts.verbose)
        .set_no_default_features(opts.no_default_features)
        .set_all_features(opts.all_features)
        .get_lints()?;

    let warnings_caused_by_diff =
        intersections::get_lints_from_diff(&clippy_lints, &diff_sections, opts.verbose);

    return_warnings(&warnings_caused_by_diff, opts.without_error)
}

fn return_warnings(lints: &[clippy::Lint], without_error: bool) -> Result<(), error::Error> {
    if lints.is_empty() {
        println!("No warnings raised by clippy::pedantic in your diff, you're good to go!");
        Ok(())
    } else {
        if without_error {
            display_warnings(&lints);
            return Ok(());
        }
        display_warnings(&lints);
        println!("Clippy::pedantic found {} warnings", lints.len());
        Err(error::Error::NotClean)
    }
}

#[cfg(test)]
mod tests {
    use crate::clippy::Lint;
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
