extern crate clap;
use clap::{App, Arg};

mod clippy;
mod error;
mod git;

fn lines_in_range(span: &clippy::Span, diff: &git::Section) -> bool {
    span.line_start >= diff.line_start && span.line_start <= diff.line_end
        || span.line_end >= diff.line_start && span.line_end <= diff.line_end
}

fn get_lints_from_diff(
    lints: &[clippy::Lint],
    diffs: &[git::Section],
    _verbose: bool,
) -> Vec<clippy::Lint> {
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

fn main() -> Result<(), error::Error> {
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
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Set the verbosity level"),
        )
        .get_matches();

    let verbose = matches.is_present("verbose");
    let target_branch = matches.value_of("branch").unwrap_or("master");

    println!("Getting diff against target {}", target_branch);
    let diff_sections = git::get_diff_sections(target_branch, verbose)?;
    println!("Running clippy");
    let clippy_lints = clippy::get_clippy_lints(verbose)?;

    let warnings_caused_by_diff = get_lints_from_diff(&clippy_lints, &diff_sections, verbose);
    if warnings_caused_by_diff.is_empty() {
        println!("No warnings raised by clippy::pedantic in your diff, you're good to go!");
        Ok(())
    } else {
        let total_warnings = warnings_caused_by_diff.len();
        for w in warnings_caused_by_diff {
            if let Some(m) = w.message {
                for l in m.rendered.split('\n') {
                    println!("{}", l);
                }
            }
        }
        println!("Clippy::pedantic found {} warnings", total_warnings);
        Err(error::Error::NotClean)
    }
}
