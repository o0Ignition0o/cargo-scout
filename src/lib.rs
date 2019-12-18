pub use error::Error;
pub use linter::{Lint, Linter};
pub use project::Config;

use linter::clippy::Clippy;
use project::cargo::Config as CargoConfig;
use scout::Builder;
use std::env;

mod error;
mod git;
mod linter;
mod project;
mod scout;

pub struct ScoutOptions {
    /// Set the verbosity level
    pub verbose: bool,
    /// Pass the no default features flag to clippy
    pub no_default_features: bool,
    /// Pass the all features flag to clippy
    pub all_features: bool,
    /// Set the target branch
    pub branch: String,
    /// Pass the path of the `Cargo.toml` file
    pub cargo_toml: String,
    /// Enable nightly features (e.g. get lints even after the build has already been done.)
    pub preview: bool,
}

pub fn run(opts: ScoutOptions) -> Result<Vec<Lint>, Error> {
    println!("Getting diff against target {}", opts.branch);
    let diff_sections = git::get_sections(env::current_dir()?, &opts.branch)?;

    println!("Checking Cargo manifest");
    let mut project = CargoConfig::from_manifest_path(opts.cargo_toml)?;
    project.set_all_features(opts.all_features);
    project.set_no_default_features(opts.no_default_features);

    let mut clippy_linter = Clippy::new();
    clippy_linter
        .set_verbose(opts.verbose)
        .set_no_default_features(opts.no_default_features)
        .set_all_features(opts.all_features)
        .set_preview(opts.preview);

    let mut scout_builder = Builder::new();
    scout_builder
        .set_project_config(project)
        .set_linter(clippy_linter);
    println!("Running clippy");
    let scout = scout_builder.build()?;
    scout.run_for_diff(&diff_sections)
}
