use serde::Deserialize;
use std::path::PathBuf;

pub mod clippy;

pub trait Linter {
    fn get_lints(&self, working_dir: PathBuf) -> Result<Vec<Lint>, crate::error::Error>;
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct LintCode {
    pub code: String,
    pub explanation: String,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct LintSpan {
    pub file_name: String,
    /// The line where the lint should be reported
    ///
    /// GitHub provides a line_start and a line_end.
    /// We should use the line_start in case of multi-line lints.
    /// (Why?)
    pub line_start: usize,
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct Lint {
    /// The lint message
    ///
    /// Example:
    ///
    /// unused variable: `count`
    pub package_id: String,
    pub src_path: Option<String>,
    pub message: Option<Message>,
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct Message {
    pub rendered: String,
    pub spans: Vec<Span>,
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct Span {
    pub file_name: String,
    pub line_start: u32,
    pub line_end: u32,
}
