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
    #[structopt(
        short = "b",
        long = "branch",
        value_name = "branch",
        default_value = "master"
    )]
    /// Set the target branch
    branch: String,

    #[structopt(short = "t", long = "cargo-toml")]
    /// Pass the path of the `Cargo.toml` file
    cargo_toml: Option<String>,
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
    let path = opts
        .cargo_toml
        .unwrap_or_else(|| String::from("./Cargo.toml"));
    let manifest = cargo::Parser::from_manifest_path(path)?;
    if manifest.is_workspace() {
        println!("Running in workspace, please note feature flags are not supported yet.");
    }
    println!("Running clippy");
    let clippy_lints = clippy::Linter::new()
        .set_verbose(opts.verbose)
        .set_no_default_features(opts.no_default_features)
        .get_lints()?;

    let warnings_caused_by_diff =
        intersections::get_lints_from_diff(&clippy_lints, &diff_sections, opts.verbose);
    if warnings_caused_by_diff.is_empty() {
        println!("No warnings raised by clippy::pedantic in your diff, you're good to go!");
        Ok(())
    } else {
        display_warnings(&warnings_caused_by_diff);
        println!(
            "Clippy::pedantic found {} warnings",
            warnings_caused_by_diff.len()
        );
        Err(error::Error::NotClean)
    }
}
