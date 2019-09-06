extern crate clap;
use clap::{App, Arg};
use std::process::Command;

#[derive(Debug)]
enum ScoutError {
    CommandError(String),
    Utf8Error(std::string::FromUtf8Error),
    NotClean
}

impl From<std::string::FromUtf8Error> for ScoutError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        ScoutError::Utf8Error(err)
    }
}

impl From<String> for ScoutError {
    fn from(err: String) -> Self {
        ScoutError::CommandError(err)
    }
}

fn git_diff(target: &str) -> Result<String, ScoutError> {
    let cmd_output = Command::new("git")
        .args(&["diff", target])
        .output()
        .expect("Could not run git command.");
    if !cmd_output.status.success() {
        Err(String::from_utf8(cmd_output.stderr)?.into())
    } else {
        Ok(String::from_utf8(cmd_output.stdout)?)
    }
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

    let diff = git_diff(target)?;
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

    println!("{:#?}", diff);

    let mut warnings_caused_by_diff: Vec<&str> = Vec::new();
    /*for warning in clippy_pedantic_output {
        if caused_by_diff(warning, diffs) {
            warnings_caused_by_diff.push(warning);
        }
    }*/

    if !warnings_caused_by_diff.is_empty() {
        println!("No warnings raised by clippy::pedantic in your diff, you're good to go!");
        Ok(())
    } else {
        println!(
            "Clippy::pedantic found {} warnings",
            warnings_caused_by_diff.len()
        );
        for w in warnings_caused_by_diff {
            println!("{}", w);
        }
        Err(ScoutError::NotClean)
    }
}
