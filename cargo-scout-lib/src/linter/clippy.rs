use crate::linter;
use crate::utils::get_absolute_file_path;
use cargo_scout_macros::{error, info};
use colored::Colorize;
use serde::Deserialize;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct Clippy {
    verbose: bool,
    no_default_features: bool,
    all_features: bool,
    features: Option<String>,
    preview: bool,
}

#[derive(Deserialize, Clone)]
/// A `Linter`s output is a `Vec<Lint>`
struct Lint {
    /// The package id
    /// Example:
    /// "cargo-scout-lib".to_string()
    package_id: String,
    /// The file the lint was reported on
    /// Example:
    /// Some("src/lib.rs".to_string())
    src_path: Option<String>,
    /// The message structure
    message: Option<Message>,
}

#[derive(Deserialize, Clone)]
/// This struct contains the message output,
/// and a `Vec<Span>` with the message location
struct Message {
    /// The message string
    /// Example:
    /// unused variable `count`
    rendered: String,
    /// The file names and lines the lint
    /// was reported on
    spans: Vec<Span>,
}

#[derive(Deserialize, Clone)]
/// A `Span` has a file name, a start and an end line
struct Span {
    file_name: String,
    line_start: u32,
    line_end: u32,
}

impl linter::Linter for Clippy {
    fn lints(
        &self,
        working_dir: impl Into<PathBuf>,
    ) -> Result<Vec<linter::Lint>, crate::error::Error> {
        let working_dir = working_dir.into();
        info!(
            "[Clippy] - getting lints for directory {}",
            &working_dir.to_str().unwrap_or("<no directory>")
        );
        self.clippy(working_dir)
            .map(|clippy_output| lints(clippy_output.as_ref()))
    }
}

impl Clippy {
    pub fn set_verbose(&mut self, verbose: bool) -> &mut Self {
        self.verbose = verbose;
        self
    }

    pub fn set_no_default_features(&mut self, no_default_features: bool) -> &mut Self {
        self.no_default_features = no_default_features;
        self
    }

    pub fn set_all_features(&mut self, all_features: bool) -> &mut Self {
        self.all_features = all_features;
        self
    }

    pub fn set_features(&mut self, features: Option<String>) -> &mut Self {
        self.features = features;
        self
    }

    pub fn set_preview(&mut self, preview: bool) -> &mut Self {
        self.preview = preview;
        self
    }

    fn command_parameters(&self) -> Vec<&str> {
        let mut params = if self.preview {
            vec![
                "+nightly",
                "clippy-preview",
                "-Z",
                "unstable-options",
                "--message-format",
                "json",
            ]
        } else {
            vec!["clippy", "--message-format", "json"]
        };
        if self.verbose {
            params.push("--verbose");
        }
        if self.no_default_features {
            params.push("--no-default-features");
        }
        if self.all_features {
            params.push("--all-features");
        }
        if let Some(features) = &self.features {
            params.append(&mut vec!["--features", features]);
        }
        params.append(&mut vec![
            "--tests",
            "--",
            "-W",
            "clippy::all",
            "-W",
            "clippy::pedantic",
        ]);
        params
    }

    fn envs(&self) -> Vec<(&str, &str)> {
        let mut envs = vec![];
        if self.verbose {
            envs.push(("RUST_BACKTRACE", "full"));
        }
        envs
    }

    // Skipped from code coverage
    // because an external command
    // cannot be easily unit tested
    #[cfg(not(tarpaulin_include))]
    fn clippy(&self, path: impl AsRef<Path>) -> Result<String, crate::error::Error> {
        let clippy_pedantic_output = Command::new("cargo")
            .current_dir(path)
            .args(self.command_parameters())
            .envs(self.envs())
            .output()
            .expect("failed to run clippy pedantic");

        if self.verbose {
            info!(
                "{}",
                String::from_utf8(clippy_pedantic_output.stdout.clone())?
            );
        }
        if clippy_pedantic_output.status.success() {
            Ok(String::from_utf8(clippy_pedantic_output.stdout)?)
        } else if self.verbose {
            error!("Clippy run failed");
            info!("cleaning and building with full backtrace");
            let _ = Command::new("cargo")
                .args(&["clean"])
                .envs(self.envs())
                .output()
                .expect("failed to start cargo clean");
            let build = Command::new("cargo")
                .args(&["build"])
                .envs(self.envs())
                .output()
                .expect("failed to start cargo build");
            if build.status.success() {
                Err(crate::error::Error::Command(String::from_utf8(
                    build.stdout,
                )?))
            } else {
                io::stdout().write_all(&build.stdout)?;
                Err(crate::error::Error::Command(String::from_utf8(
                    build.stderr,
                )?))
            }
        } else {
            Err(crate::error::Error::Command(String::from_utf8(
                clippy_pedantic_output.stderr,
            )?))
        }
    }
}

#[must_use]
fn lints(clippy_output: &str) -> Vec<linter::Lint> {
    let mut lints = Vec::new();

    let clippy_messages: Vec<Message> = clippy_output
        .lines()
        .filter(|l| l.starts_with('{'))
        .filter_map(|line| {
            if let Ok(lint) = serde_json::from_str::<Lint>(line) {
                lint.message
            } else {
                None
            }
        })
        .filter(|message: &Message| !message.spans.is_empty())
        .collect();

    for c in clippy_messages {
        for s in c.spans {
            if let Ok(path) = get_absolute_file_path(s.file_name) {
                lints.push(linter::Lint {
                    message: c.rendered.clone(),
                    location: linter::Location {
                        path,
                        lines: [s.line_start, s.line_end],
                    },
                })
            }
        }
    }
    lints
}

#[cfg(test)]
mod tests {
    use super::{get_absolute_file_path, lints, Clippy};
    use crate::error::Error;

    #[test]
    fn test_set_verbose() {
        let mut linter = Clippy::default();
        assert_eq!(false, linter.verbose);

        let l2 = linter.set_verbose(true);
        assert_eq!(true, l2.verbose);

        let l3 = l2.set_verbose(false);
        assert_eq!(false, l3.verbose);
    }

    #[test]
    fn test_get_envs() {
        let mut linter = Clippy::default();
        let mut expected_envs = vec![];
        assert_eq!(expected_envs, linter.envs());

        let verbose_linter = linter.set_verbose(true);
        expected_envs.push(("RUST_BACKTRACE", "full"));
        assert_eq!(expected_envs, verbose_linter.envs());
    }

    #[test]
    fn test_get_command_parameters() {
        let mut linter = Clippy::default();
        let expected_command_parameters = vec![
            "clippy",
            "--message-format",
            "json",
            "--tests",
            "--",
            "-W",
            "clippy::all",
            "-W",
            "clippy::pedantic",
        ];

        assert_eq!(expected_command_parameters, linter.command_parameters());

        let verbose_linter = linter.set_verbose(true);
        assert!(verbose_linter.command_parameters().contains(&"--verbose"));

        let no_default_features_linter = linter.set_verbose(false).set_no_default_features(true);
        assert!(no_default_features_linter
            .command_parameters()
            .contains(&"--no-default-features"));

        let all_features_linter = linter
            .set_verbose(false)
            .set_no_default_features(false)
            .set_all_features(true);
        assert!(all_features_linter
            .command_parameters()
            .contains(&"--all-features"));

        let features_linter = linter
            .set_all_features(false)
            .set_features(Some(String::from("foo bar baz")));

        assert!(features_linter.command_parameters().contains(&"--features"));
        assert!(features_linter
            .command_parameters()
            .contains(&"foo bar baz"));
    }

    #[test]
    fn test_get_command_parameters_nightly() {
        let mut nightly_linter = Clippy::default();
        let nightly_linter = nightly_linter.set_preview(true);
        assert!(nightly_linter
            .command_parameters()
            .contains(&"clippy-preview"));

        let nightly_verbose_linter = nightly_linter.set_verbose(true);
        assert!(nightly_verbose_linter
            .command_parameters()
            .contains(&"--verbose"));

        let nightly_all_features_linter = nightly_linter.set_verbose(false).set_all_features(true);
        assert!(nightly_all_features_linter
            .command_parameters()
            .contains(&"--all-features"));

        let nightly_no_default_features_linter = nightly_linter
            .set_verbose(false)
            .set_all_features(false)
            .set_no_default_features(true);
        assert!(nightly_no_default_features_linter
            .command_parameters()
            .contains(&"--no-default-features"));

        let nightly_features_linter = nightly_linter
            .set_no_default_features(false)
            .set_features(Some(String::from("foo bar baz")));
        assert!(nightly_features_linter
            .command_parameters()
            .contains(&"--features"));
        assert!(nightly_features_linter
            .command_parameters()
            .contains(&"foo bar baz"));
    }

    #[test]
    fn test_lints() -> Result<(), Error> {
        use crate::linter;
        let expected_lints = vec![linter::Lint {
            message: "this is a test lint".to_string(),
            location: linter::Location {
                path: get_absolute_file_path("test/foo/baz.rs")?,
                lines: [10, 12],
            },
        }];

        let clippy_output = r#"{"package_id": "cargo-scout","src_path": "test/foo/bar.rs","message": { "rendered": "this is a test lint","spans": [{"file_name": "test/foo/baz.rs","line_start": 10,"line_end": 12}]}}"#;

        assert_eq!(expected_lints, lints(clippy_output));
        Ok(())
    }
}
