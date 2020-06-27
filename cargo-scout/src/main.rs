use cargo_scout_lib::config::rust::CargoConfig;
use cargo_scout_lib::linter::clippy::Clippy;
use cargo_scout_lib::linter::rustfmt::RustFmt;
use cargo_scout_lib::linter::Lint;
use cargo_scout_lib::scout::Scout;
use cargo_scout_lib::vcs::git::Git;
use cargo_scout_lib::Error;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(
    name = "cargo-scout",
    author,
    about = "Leave the codebase better than when you found it."
)]
enum Command {
    Fmt(FmtOptions),
    Lint(LintOptions),
}

#[derive(Debug, StructOpt)]
struct FmtOptions {
    #[structopt(
        short = "b",
        long = "branch",
        value_name = "branch",
        default_value = "HEAD"
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

#[derive(Debug, StructOpt)]
struct LintOptions {
    #[structopt(short = "v", long = "verbose")]
    /// Set the verbosity level
    verbose: bool,
    #[structopt(long = "no-default-features")]
    /// Pass the no default features flag to clippy
    no_default_features: bool,
    #[structopt(long = "all-features")]
    /// Pass the all features flag to clippy
    all_features: bool,
    #[structopt(long = "features")]
    /// Pass features to clippy
    features: Option<String>,
    #[structopt(
        short = "b",
        long = "branch",
        value_name = "branch",
        default_value = "HEAD"
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

// There is no logic to test
#[cfg(not(tarpaulin_include))]
fn main() -> Result<(), Error> {
    match Command::from_args() {
        Command::Fmt(opts) => run_fmt(opts),
        Command::Lint(opts) => run_lint(opts),
    }
}

#[cfg(not(tarpaulin_include))]
fn run_lint(opts: LintOptions) -> Result<(), Error> {
    let fail_if_errors = opts.without_error;

    let vcs = Git::with_target(opts.branch);
    let config = CargoConfig::from_manifest_path(opts.cargo_toml)?;
    let mut linter = Clippy::default();
    linter
        .set_verbose(opts.verbose)
        .set_no_default_features(opts.no_default_features)
        .set_all_features(opts.all_features)
        .set_features(opts.features)
        .set_preview(opts.preview);
    let scout = Scout::new(vcs, config, linter);
    let relevant_lints = scout.run()?;
    return_warnings(&relevant_lints, fail_if_errors)
}

#[cfg(not(tarpaulin_include))]
fn run_fmt(opts: FmtOptions) -> Result<(), Error> {
    let fail_if_errors = opts.without_error;

    let vcs = Git::with_target(opts.branch);
    let config = CargoConfig::from_manifest_path(opts.cargo_toml)?;
    let linter = RustFmt::default();

    let scout = Scout::new(vcs, config, linter);
    let relevant_lints = scout.run()?;
    return_warnings(&relevant_lints, fail_if_errors)
}

fn return_warnings(lints: &[Lint], without_error: bool) -> Result<(), Error> {
    if lints.is_empty() {
        println!("No issues in your diff, you're good to go!");
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
        for l in w.message.split('\n') {
            println!("{}", l);
        }
    }
    if warnings.len() == 1 {
        println!("Cargo scout found a warning");
    } else {
        println!("Cargo scout found {} warnings", warnings.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cargo_scout_lib::linter::Location;
    #[test]
    fn test_return_status_with_lints() {
        let lints = vec![Lint {
            message: String::new(),
            location: Location {
                path: String::new(),
                lines: [0, 0],
            },
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
