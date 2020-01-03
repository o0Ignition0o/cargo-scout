use crate::linter::{Lint, Linter};
use std::io::{self, Write};
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

#[derive(Default)]
pub struct Clippy {
    verbose: bool,
    no_default_features: bool,
    all_features: bool,
    features: Option<String>,
    preview: bool,
}

impl Linter for Clippy {
    fn lints(&self, working_dir: PathBuf) -> Result<Vec<Lint>, crate::error::Error> {
        println!(
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
        params.append(&mut vec!["--", "-W", "clippy::pedantic"]);
        params
    }

    fn envs(&self) -> Vec<(&str, &str)> {
        let mut envs = vec![];
        if self.verbose {
            envs.push(("RUST_BACKTRACE", "full"));
        }
        envs
    }

    fn clippy(&self, path: impl AsRef<Path>) -> Result<String, crate::error::Error> {
        let clippy_pedantic_output = Command::new("cargo")
            .current_dir(path)
            .args(self.command_parameters())
            .envs(self.envs())
            .output()
            .expect("failed to run clippy pedantic");

        if self.verbose {
            println!(
                "{}",
                String::from_utf8(clippy_pedantic_output.stdout.clone())?
            );
        }
        if clippy_pedantic_output.status.success() {
            Ok(String::from_utf8(clippy_pedantic_output.stdout)?)
        } else if self.verbose {
            println!("Clippy run failed");
            println!("cleaning and building with full backtrace");
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
fn lints(clippy_output: &str) -> Vec<Lint> {
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
    use super::*;
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
            "--",
            "-W",
            "clippy::pedantic",
        ];

        assert_eq!(expected_command_parameters, linter.command_parameters());

        let verbose_linter = linter.set_verbose(true);
        let verbose_expected_command_parameters = vec![
            "clippy",
            "--message-format",
            "json",
            "--verbose",
            "--",
            "-W",
            "clippy::pedantic",
        ];
        assert_eq!(
            verbose_expected_command_parameters,
            verbose_linter.command_parameters()
        );

        let no_default_features_linter = linter.set_verbose(false).set_no_default_features(true);
        let no_default_features_expected_command_parameters = vec![
            "clippy",
            "--message-format",
            "json",
            "--no-default-features",
            "--",
            "-W",
            "clippy::pedantic",
        ];
        assert_eq!(
            no_default_features_expected_command_parameters,
            no_default_features_linter.command_parameters()
        );

        let all_features_linter = linter
            .set_verbose(false)
            .set_no_default_features(false)
            .set_all_features(true);
        let all_features_expected_command_parameters = vec![
            "clippy",
            "--message-format",
            "json",
            "--all-features",
            "--",
            "-W",
            "clippy::pedantic",
        ];
        assert_eq!(
            all_features_expected_command_parameters,
            all_features_linter.command_parameters()
        );

        let features_linter = linter
            .set_all_features(false)
            .set_features(Some(String::from("foo bar baz")));
        let features_expected_command_parameters = vec![
            "clippy",
            "--message-format",
            "json",
            "--features",
            "foo bar baz",
            "--",
            "-W",
            "clippy::pedantic",
        ];
        assert_eq!(
            features_expected_command_parameters,
            features_linter.command_parameters()
        );

        let mut nightly_linter = Clippy::default();
        let nightly_linter = nightly_linter.set_preview(true);
        let expected_command_parameters = vec![
            "+nightly",
            "clippy-preview",
            "-Z",
            "unstable-options",
            "--message-format",
            "json",
            "--",
            "-W",
            "clippy::pedantic",
        ];
        assert_eq!(
            expected_command_parameters,
            nightly_linter.command_parameters()
        );

        let nightly_verbose_linter = nightly_linter.set_verbose(true);
        let verbose_expected_command_nightly_parameters = vec![
            "+nightly",
            "clippy-preview",
            "-Z",
            "unstable-options",
            "--message-format",
            "json",
            "--verbose",
            "--",
            "-W",
            "clippy::pedantic",
        ];
        assert_eq!(
            verbose_expected_command_nightly_parameters,
            nightly_verbose_linter.command_parameters()
        );

        let nightly_all_features_linter = nightly_linter.set_verbose(false).set_all_features(true);
        let all_features_expected_command_nightly_parameters = vec![
            "+nightly",
            "clippy-preview",
            "-Z",
            "unstable-options",
            "--message-format",
            "json",
            "--all-features",
            "--",
            "-W",
            "clippy::pedantic",
        ];
        assert_eq!(
            all_features_expected_command_nightly_parameters,
            nightly_all_features_linter.command_parameters()
        );

        let nightly_no_default_features_linter = nightly_linter
            .set_verbose(false)
            .set_all_features(false)
            .set_no_default_features(true);
        let no_default_features_expected_command_nightly_parameters = vec![
            "+nightly",
            "clippy-preview",
            "-Z",
            "unstable-options",
            "--message-format",
            "json",
            "--no-default-features",
            "--",
            "-W",
            "clippy::pedantic",
        ];
        assert_eq!(
            no_default_features_expected_command_nightly_parameters,
            nightly_no_default_features_linter.command_parameters()
        );

        let nightly_features_linter = nightly_linter
            .set_no_default_features(false)
            .set_features(Some(String::from("foo bar baz")));
        let features_expected_command_nightly_parameters = vec![
            "+nightly",
            "clippy-preview",
            "-Z",
            "unstable-options",
            "--message-format",
            "json",
            "--features",
            "foo bar baz",
            "--",
            "-W",
            "clippy::pedantic",
        ];
        assert_eq!(
            features_expected_command_nightly_parameters,
            nightly_features_linter.command_parameters()
        );
    }
    #[test]
    fn test_lints() {
        use crate::linter::{Message, Span};
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
