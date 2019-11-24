# cargo-scout
  <p>
    <a href="CODE_OF_CONDUCT.md">
      <img src="https://img.shields.io/badge/Contributor%20Covenant-v1.4%20adopted-ff69b4.svg" alt="Contributor Covenant">
  </a>
    <a href="https://travis-ci.com/o0Ignition0o/cargo-scout">
      <img src="https://travis-ci.com/o0Ignition0o/cargo-scout.svg?branch=master" alt="Travis Build Status">
  </a>
  <a href="LICENSE-APACHE">
    <img
    src="https://img.shields.io/badge/license-apache2-green.svg" alt="MPL 2.0 License">
  </a>
  <a href="LICENSE-MIT">
    <img
    src="https://img.shields.io/badge/license-mit-blue.svg" alt="MIT License">
  </a>
</p>


# Leave this world a little better than you found it

A cargo subcommand to get [clippy::pedantic](https://github.com/rust-lang/rust-clippy#clippy) lints for the changes you have made in a codebase.


## Current Status

cargo-scout is experimental and in a very rough draft for now.

The current minimum Rust version supported is 1.37 stable.

## Prerequisites
Git: In order to compute a set of changes, it requires a project running git.

cargo-clippy as a subcommand: Instructions can be found [in the clippy repository](https://github.com/rust-lang/rust-clippy#as-a-cargo-subcommand-cargo-clippy).

## How to install
```bash
$ cargo install cargo-scout
```

## How to run it
Open a shell, go to the project you would like to run the command in, and run cargo-scout, with an optional target branch:
```bash
$ cargo-scout # Diff with master
$ cargo-scout -b <branch_name> # Diff with the target branch you chose.
```
A git diff will be queried and clippy will be run as well, searching for lints that may apply to your diff.

If some lints can apply, the command execution will error out. This design decision has been made so you can put it in your CI pipeline at some point (but please wait for 1.0 release ^^').


## Code of Conduct

We have a Code of Conduct so as to create a more enjoyable community and
work environment. Please see the [CODE_OF_CONDUCT](CODE_OF_CONDUCT.md)
file for more details.

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

Dual MIT/Apache2 is strictly more permissive