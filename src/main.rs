extern crate clap;
use clap::{App, Arg};

mod clippy;
mod error;
mod git;
mod intersections;

fn display_warnings(warnings: &[clippy::Lint]) {
    for w in warnings {
        if let Some(m) = &w.message {
            for l in m.rendered.split('\n') {
                println!("{}", l);
            }
        }
    }
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
    let diff_sections = git::Parser::new()
        .set_verbose(verbose)
        .get_sections(target_branch)?;
    println!("Running clippy");
    let clippy_lints = clippy::Linter::new().set_verbose(verbose).get_lints()?;

    let warnings_caused_by_diff =
        intersections::get_lints_from_diff(&clippy_lints, &diff_sections, verbose);
    if warnings_caused_by_diff.is_empty() {
        println!("No warnings raised by clippy::pedantic in your diff, you're good to go!");
        Ok(())
    } else {
        display_warnings(&warnings_caused_by_diff);
        println!(
            "Clippy::pedantic found {} warnings",
            warnings_caused_by_diff.len()
        );
        Err(error::Error::NotClean)
    }
}
