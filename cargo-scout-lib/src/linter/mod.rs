use std::path::PathBuf;

pub mod clippy;
pub mod rustfmt;

pub trait Linter {
    #[allow(clippy::missing_errors_doc)]
    fn lints(&self, working_dir: impl Into<PathBuf>) -> Result<Vec<Lint>, crate::error::Error>;
}

/// This struct contains the lint,
/// It may contain a message, and a location.
#[derive(PartialEq, Clone, Debug)]
pub struct Lint {
    /// The message string
    /// Example:
    /// unused variable `count`
    pub message: String,
    /// The file names and lines the lint
    /// was reported on
    pub location: Location,
}

/// A `Location` has a file name, a start and an end line
#[derive(PartialEq, Clone, Debug)]
pub struct Location {
    pub path: String,
    pub lines: [u32; 2],
}
