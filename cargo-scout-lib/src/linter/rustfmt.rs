use crate::error::Error;
use crate::healer::Healer;
use crate::linter::{Lint, Linter, Location};
use crate::utils::get_absolute_file_path;
use crate::utils::get_relative_file_path;
use serde::{Deserialize, Serialize};
use serde_json;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Default)]
pub struct RustFmt {}

#[derive(Serialize)]
struct FmtLocation {
    file: String,
    range: [u32; 2],
}

impl Linter for RustFmt {
    fn lints(&self, working_dir: impl Into<PathBuf>) -> Result<Vec<Lint>, Error> {
        let working_dir = working_dir.into();
        println!(
            "[RustFmt] - checking format for directory {}",
            &working_dir.to_str().unwrap_or("<no directory>")
        );
        let rustfmt_output = Self::fmt(working_dir)?;
        lints(&rustfmt_output)
    }
}

impl Healer for RustFmt {
    // Skipped from code coverage
    // because an external command
    // cannot be easily unit tested
    #[cfg_attr(tarpaulin, skip)]
    fn heal(&self, lints: Vec<Lint>) -> Result<(), crate::error::Error> {
        let l = &lints_as_json(&lints)?;
        let fmt_fix = Command::new("cargo")
            .args(&[
                "+nightly",
                "fmt",
                "--",
                "--unstable-features",
                "--file-lines",
                l,
                "--skip-children",
            ])
            .output()
            .expect("failed to run rustfmt");

        if fmt_fix.status.success() {
            Ok(())
        } else {
            Err(crate::error::Error::Command(String::from_utf8(
                fmt_fix.stderr,
            )?))
        }
    }
}

fn lints_as_json(lints: &[Lint]) -> Result<String, crate::error::Error> {
    let locations: Vec<FmtLocation> = lints
        .iter()
        .filter_map(|l| {
            if let Ok(file) = get_relative_file_path(l.location.path.clone()) {
                Some(FmtLocation {
                    file,
                    range: l.location.lines,
                })
            } else {
                None
            }
        })
        .collect();
    Ok(serde_json::to_string(&locations)?)
}

impl RustFmt {
    fn command_parameters() -> Vec<&'static str> {
        vec!["+nightly", "fmt", "--", "--emit", "json"]
    }

    // Skipped from code coverage
    // because an external command
    // cannot be easily unit tested
    #[cfg_attr(tarpaulin, skip)]
    fn fmt(path: impl AsRef<Path>) -> Result<String, Error> {
        let fmt_output = Command::new("cargo")
            .current_dir(path)
            .args(Self::command_parameters())
            .output()
            .expect("failed to run cargo fmt");

        if fmt_output.status.success() {
            Ok(String::from_utf8(fmt_output.stdout)?)
        } else {
            Err(Error::Command(String::from_utf8(fmt_output.stderr)?))
        }
    }
}

#[derive(Deserialize, Debug)]
struct FmtLint {
    name: String,
    mismatches: Vec<FmtMismatch>,
}

#[derive(Deserialize, Debug)]
struct FmtMismatch {
    original_begin_line: u32,
    original_end_line: u32,
    original: String,
    expected: String,
}

fn lints(fmt_output: &str) -> Result<Vec<Lint>, Error> {
    let mut lints = Vec::new();
    let fmt_lints: Vec<FmtLint> = serde_json::from_str(fmt_output)?;
    for fmt_lint in fmt_lints {
        lints.append(
            &mut fmt_lint
                .mismatches
                .iter()
                .filter_map(|mismatch| {
                    if let Ok(path) = get_absolute_file_path(fmt_lint.name.clone()) {
                        Some(Lint {
                            message: display_mismatch(mismatch, &path),
                            location: Location {
                                path,
                                lines: [mismatch.original_begin_line, mismatch.original_end_line],
                            },
                        })
                    } else {
                        None
                    }
                })
                .collect::<Vec<Lint>>(),
        );
    }
    Ok(lints)
}

fn display_mismatch(mismatch: &FmtMismatch, path: &str) -> String {
    if mismatch.original_begin_line == mismatch.original_end_line {
        format!(
            "Diff in {} at line {}:\n-{}\n+{}\n",
            path, mismatch.original_begin_line, mismatch.original, mismatch.expected
        )
    } else {
        format!(
            "Diff in {} between lines {} and {}:\n{}\n{}\n",
            path,
            mismatch.original_begin_line,
            mismatch.original_end_line,
            mismatch
                .original
                .lines()
                .map(|line| format!("-{}", line))
                .collect::<Vec<String>>()
                .join("\n"),
            mismatch
                .expected
                .lines()
                .map(|line| format!("+{}", line))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_parameters() {
        assert_eq!(
            vec!["+nightly", "fmt", "--", "--emit", "json"],
            RustFmt::command_parameters()
        );
    }

    #[test]
    fn test_display_mismatch_one_line() -> Result<(), Error> {
        let mismatch = FmtMismatch {
            original_begin_line: 1,
            original_end_line: 1,
            original: "    this is a test mismatch".to_string(),
            expected: "this is a test mismatch".to_string(),
        };

        let path = get_absolute_file_path("src/foo/bar.txt")?;

        let expected_display = format!(
            "Diff in {} at line 1:\n-    this is a test mismatch\n+this is a test mismatch\n",
            path
        );
        let actual_display = display_mismatch(&mismatch, &path);
        assert_eq!(expected_display, actual_display);
        Ok(())
    }

    #[test]
    fn test_display_mismatch_several_lines() -> Result<(), Error> {
        let mismatch = FmtMismatch {
            original_begin_line: 1,
            original_end_line: 2,
            original: "    this is a test mismatch\n  the indent is wrong".to_string(),
            expected: "this is a test mismatch\nthe indent is wrong".to_string(),
        };

        let path = get_absolute_file_path("src/foo/bar.txt")?;

        let expected_display = format!("Diff in {} between lines 1 and 2:\n-    this is a test mismatch\n-  the indent is wrong\n+this is a test mismatch\n+the indent is wrong\n", path);
        let actual_display = display_mismatch(&mismatch, &path);
        assert_eq!(expected_display, actual_display);
        Ok(())
    }

    #[test]
    fn test_lints() -> Result<(), crate::error::Error> {
        let fmt_output = r#"[{"name":"cargo-scout/cargo-scout-lib/src/lib.rs","mismatches":[{"original_begin_line":1,"original_end_line":1,"expected_begin_line":1,"expected_end_line":1,"original":"    pub mod config;","expected":"pub mod config;"}]}]"#;

        let path = get_absolute_file_path("cargo-scout/cargo-scout-lib/src/lib.rs")?;
        let expected_lints = vec![Lint {
            location: Location {
                lines: [1, 1],
                path: path.clone(),
            },
            message: format!(
                "Diff in {} at line 1:\n-    pub mod config;\n+pub mod config;\n",
                path
            ),
        }];

        let actual_lints = lints(fmt_output).unwrap();

        assert_eq!(expected_lints, actual_lints);

        Ok(())
    }

    #[test]
    fn test_lints_as_json() -> Result<(), crate::error::Error> {
        let expected_output = r#"[{"file":"src/lib.rs","range":[1,2]},{"file":"foo_bar.rs","range":[11,19]},{"file":"baz.rs","range":[1,1]}]"#;

        let lints_to_transform = vec![
            Lint {
                message: String::new(),
                location: Location {
                    path: get_absolute_file_path("src/lib.rs")?,
                    lines: [1, 2],
                },
            },
            Lint {
                message: String::new(),
                location: Location {
                    path: get_absolute_file_path("foo_bar.rs")?,
                    lines: [11, 19],
                },
            },
            Lint {
                message: String::new(),
                location: Location {
                    path: get_absolute_file_path("baz.rs")?,
                    lines: [1, 1],
                },
            },
            Lint {
                message: "this one won't parse because the path is not absolute".to_string(),
                location: Location {
                    path: "wont_pass.rs".to_string(),
                    lines: [42, 42],
                },
            },
        ];

        let actual_output = lints_as_json(&lints_to_transform)?;
        assert_eq!(expected_output, actual_output);
        Ok(())
    }
}
