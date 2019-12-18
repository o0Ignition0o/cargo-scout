use cargo_scout::{run, Error, Lint, ScoutOptions};
pub use structopt::StructOpt;

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
    #[structopt(short = "p", long = "preview")]
    /// Enable nightly features (e.g. get lints even after the build has already been done.)
    preview: bool,
}

impl From<Options> for ScoutOptions {
    fn from(o: Options) -> Self {
        Self {
            verbose: o.verbose,
            no_default_features: o.no_default_features,
            all_features: o.all_features,
            branch: o.branch,
            cargo_toml: o.cargo_toml,
            preview: o.preview,
        }
    }
}

fn main() -> Result<(), Error> {
    let opts = Options::from_args();
    let fail_if_errors = opts.without_error;
    let relevant_lints = run(opts.into())?;
    return_warnings(&relevant_lints, fail_if_errors)
}

fn return_warnings(lints: &[Lint], without_error: bool) -> Result<(), Error> {
    if lints.is_empty() {
        println!("No warnings raised by clippy::pedantic in your diff, you're good to go!");
        Ok(())
    } else {
        display_warnings(&lints);
        if without_error {
            Ok(())
        } else {
            Err(Error::NotClean)
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
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
