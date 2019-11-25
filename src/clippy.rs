use serde::Deserialize;
use std::process::Command;

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
    pub line_start: i32,
    pub line_end: i32,
}

pub fn get_clippy_lints(verbose: bool) -> Result<Vec<Lint>, crate::error::Error> {
    clippy(verbose).map(|output| lints(&output))
}

pub fn clippy(verbose: bool) -> Result<String, crate::error::Error> {
    let clippy_pedantic_output = if verbose {
        Command::new("cargo")
            .args(&[
                "clippy",
                "--verbose",
                "--message-format",
                "json",
                "--",
                "-W",
                "clippy::pedantic",
            ])
            .output()
            .expect("failed to run clippy pedantic")
    } else {
        Command::new("cargo")
            .args(&[
                "clippy",
                "--message-format",
                "json",
                "--",
                "-W",
                "clippy::pedantic",
            ])
            .output()
            .expect("failed to run clippy pedantic")
    };
    if verbose {
        println!(
            "{}",
            String::from_utf8(clippy_pedantic_output.stdout.clone())?
        );
    }
    if clippy_pedantic_output.status.success() {
        Ok(String::from_utf8(clippy_pedantic_output.stdout)?)
    } else {
        Err(String::from_utf8(clippy_pedantic_output.stderr)?.into())
    }
}

pub fn lints(clippy_output: &str) -> Vec<Lint> {
    clippy_output
        .lines()
        .filter(|l| l.starts_with('{'))
        .filter_map(|line| serde_json::from_str(line).ok())
        .filter(|lint: &Lint| {
            if let Some(m) = &lint.message {
                !m.spans.is_empty()
            } else {
                false
            }
        })
        .collect::<Vec<Lint>>()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_lints() {
        use crate::clippy::{lints, Lint, Message, Span};
        let expected_lints = vec![Lint {
            package_id: "cargo-scout".to_string(),
            src_path: Some("test/foo/bar.rs".to_string()),
            message: Some(Message {
                rendered: "this is a test lint".to_string(),
                spans: vec![Span {
                    file_name: "test/foo/baz.rs".to_string(),
                    line_start: 10,
                    line_end: 12,
                }],
            }),
        }];

        let clippy_output = r#"{"package_id": "cargo-scout","src_path": "test/foo/bar.rs","message": { "rendered": "this is a test lint","spans": [{"file_name": "test/foo/baz.rs","line_start": 10,"line_end": 12}]}}"#;

        assert_eq!(expected_lints, lints(clippy_output));
    }
}
