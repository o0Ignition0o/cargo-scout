extern crate clap;
use clap::{App, Arg};
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
    pub message: Option<ClippyMessage>,
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct ClippyMessage {
    pub rendered: String,
    pub spans: Vec<Span>,
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct Span {
    file_name: String,
    line_start: i32,
    line_end: i32,
}

#[derive(Debug)]
enum ScoutError {
    CommandError(String),
    Utf8Error(std::string::FromUtf8Error),
    JsonError(serde_json::Error),
    NotClean,
}

impl From<std::string::FromUtf8Error> for ScoutError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        ScoutError::Utf8Error(err)
    }
}

impl From<serde_json::Error> for ScoutError {
    fn from(err: serde_json::Error) -> Self {
        ScoutError::JsonError(err)
    }
}

impl From<String> for ScoutError {
    fn from(err: String) -> Self {
        ScoutError::CommandError(err)
    }
}

fn git_diff(target: &str) -> Result<String, ScoutError> {
    let cmd_output = Command::new("git")
        .args(&["diff", "-u", target])
        .output()
        .expect("Could not run git command.");
    if !cmd_output.status.success() {
        Err(String::from_utf8(cmd_output.stderr)?.into())
    } else {
        Ok(String::from_utf8(cmd_output.stdout)?)
    }
}

#[derive(Debug)]
pub struct Section {
    file_name: String,
    line_start: i32,
    line_end: i32,
}

#[derive(Debug)]
pub struct SectionBuilder {
    file_name: Option<String>,
    line_start: Option<i32>,
    line_end: Option<i32>,
}

impl SectionBuilder {
    pub fn new() -> Self {
        SectionBuilder {
            file_name: None,
            line_start: None,
            line_end: None,
        }
    }

    pub fn file_name(&mut self, file_name: String) {
        self.file_name = Some(file_name);
    }

    pub fn line_start(&mut self, line_start: i32) {
        self.line_start = Some(line_start);
    }

    pub fn line_end(&mut self, line_end: i32) {
        self.line_end = Some(line_end);
    }

    pub fn build(self) -> Option<Section> {
        match (self.file_name, self.line_start, self.line_end) {
            (Some(file_name), Some(line_start), Some(line_end)) => Some(Section {
                file_name,
                line_start,
                line_end,
            }),
            _ => None,
        }
    }
}

fn get_diff_sections(git_diff: &str) -> Vec<Section> {
    let mut sections = Vec::new();
    let mut file_name = "";
    for l in git_diff.lines() {
        // Add or edit a file
        // +++ b/Cargo.lock
        if l.starts_with("+++") {
            // TODO: do something less ugly with the bounds and indexing
            file_name = l[l.find("/").unwrap() + 1..].into();
        }
        // Actual diff lines
        // @@ -33,6 +33,9 @@ version = "0.1.0"
        if l.starts_with("@@") {
            // For now, we will focus on the added lines.
            // @@ and space
            let after_ats = &l[3..];
            // space and @@
            let before_second_ats_index = &after_ats.find("@@").unwrap() - 1;
            let diff_lines = &after_ats[..before_second_ats_index];
            // -33,6 +33,9
            let (_, a) = diff_lines.split_at(diff_lines.find(" ").unwrap());
            let added = a.trim();

            let (added_start, added_span) = if let Some(index) = added[1..].find(',') {
                let (a, b) = added[1..].split_at(index);
                (a, &b[1..])
            } else {
                (added, "")
            };
            let min_line_start = added_start.parse::<i32>().unwrap();
            let mut current_section = SectionBuilder::new();
            current_section.file_name(file_name.to_string());
            current_section.line_start(min_line_start);
            current_section.line_end(min_line_start + added_span.parse::<i32>().unwrap_or(1));
            if let Some(s) = current_section.build() {
                sections.push(s);
            }
        }
    }
    sections
}

fn clippy() -> Result<String, ScoutError> {
    let clippy_pedantic_output = Command::new("cargo")
        .args(&[
            "clippy",
            "--message-format",
            "json",
            "--",
            "-W",
            "clippy::pedantic",
        ])
        .output()
        .expect("failed to run clippy pedantic");
    if !clippy_pedantic_output.status.success() {
        Err(String::from_utf8(clippy_pedantic_output.stderr)?.into())
    } else {
        Ok(String::from_utf8(clippy_pedantic_output.stdout)?)
    }
}

fn get_clippy_lints(clippy_output: &str) -> Vec<Lint> {
    clippy_output
        .lines()
        .filter(|l| l.starts_with('{'))
        .filter_map(|line| serde_json::from_str(line).ok())
        .filter(|lint: &Lint| {
            if let Some(m) = &lint.message {
                m.spans.len() != 0
            } else {
                false
            }
        })
        .collect::<Vec<Lint>>()
}

fn lines_in_range(span: &Span, diff: &Section) -> bool {
    span.line_start >= diff.line_start && span.line_start <= diff.line_end ||
    span.line_end >= diff.line_start && span.line_end <= diff.line_end
}

fn get_lints_from_diff(lints: Vec<Lint>, diffs: Vec<Section>) -> Vec<Lint> {
    let mut lints_in_diff = Vec::new();
    for diff in diffs {
        let diff_lints = lints.iter().filter(|lint| {
            if let Some(m) = &lint.message {
                for s in &m.spans {
                    // Git diff paths and clippy paths don't get along too well on Windows...
                    if s.file_name.replace("\\", "/") == diff.file_name {
                        return lines_in_range(&s, &diff);
                    }
                }
                false
            } else {
                false
            }
        });
        for l in diff_lints {
            lints_in_diff.push(l.clone());
        }
    }

    lints_in_diff
}

fn main() -> Result<(), ScoutError> {
    let matches = App::new("cargo-scout")
        .version("1.0")
        .author("o0Ignition0o <jeremy.lempereur@gmail.com>")
        .about("Leave the codebase better than when you found it.")
        .arg(
            Arg::with_name("branch")
                .short("b")
                .long("branch")
                .value_name("branch")
                .help("Set the target branch (default: master)")
                .takes_value(true),
        )
        .get_matches();

    let target = matches.value_of("branch").unwrap_or("master");

    println!("Getting diff");
    let git_diff = git_diff(target)?;
    let diff_sections = get_diff_sections(&git_diff);
    println!("Running cargo clippy, this may take a while.");
    let clippy_output = clippy()?;
    let lints = get_clippy_lints(&clippy_output);
    let warnings_caused_by_diff = get_lints_from_diff(lints, diff_sections);

    if warnings_caused_by_diff.is_empty() {
        println!("No warnings raised by clippy::pedantic in your diff, you're good to go!");
        Ok(())
    } else {
        let total_warnings = warnings_caused_by_diff.len();
        for w in warnings_caused_by_diff {
            if let Some(m) = w.message {
                for l in m.rendered.split("\n") {
                    println!("{}", l);
                }
            }
        }
        println!(
            "Clippy::pedantic found {} warnings",
            total_warnings
        );
        Err(ScoutError::NotClean)
    }
}
