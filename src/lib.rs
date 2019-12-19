use scout::Builder;
use std::env;

pub use error::Error;
pub use linter::clippy::Clippy;
pub use linter::{Lint, Linter};
pub use project::Config;
mod error;
mod git;
pub mod linter;
pub mod project;
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

pub fn run<C: Config, L: Linter>(
    git_branch: &str,
    config: C,
    linter: L,
) -> Result<Vec<Lint>, Error> {
    println!("Getting diff against target {}", git_branch);
    let diff_sections = git::get_sections(env::current_dir()?, &git_branch)?;
    let mut scout_builder = Builder::new();
    scout_builder.set_project_config(config).set_linter(linter);
    println!("Running clippy");
    let scout = scout_builder.build()?;
    scout.run_for_diff(&diff_sections)
}
