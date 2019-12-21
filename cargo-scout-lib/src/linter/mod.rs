use serde::Deserialize;
use std::path::PathBuf;

pub mod clippy;

pub trait Linter {
    fn lints(&self, working_dir: PathBuf) -> Result<Vec<Lint>, crate::error::Error>;
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
/// A `Linter`s output is a `Vec<Lint>`
pub struct Lint {
    /// The package id
    /// Example:
    /// "cargo-scout-lib".to_string()
    pub package_id: String,
    /// The file the lint was reported on
    /// Example:
    /// Some("src/lib.rs".to_string())
    pub src_path: Option<String>,
    /// The message structure
    pub message: Option<Message>,
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
/// This struct contains the message output,
/// and a `Vec<Span>` with the message location
pub struct Message {
    /// The message string
    /// Example:
    /// unused variable `count`
    pub rendered: String,
    /// The file names and lines the lint
    /// was reported on
    pub spans: Vec<Span>,
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
/// A `Span` has a file name, a start and an end line
pub struct Span {
    pub file_name: String,
    pub line_start: u32,
    pub line_end: u32,
}
