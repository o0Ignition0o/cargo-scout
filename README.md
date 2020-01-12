# cargo-scout
  <p>
    <a href="CODE_OF_CONDUCT.md">
      <img src="https://img.shields.io/badge/Contributor%20Covenant-v1.4%20adopted-ff69b4.svg" alt="Contributor Covenant">
  </a>
    <a href="https://travis-ci.com/o0Ignition0o/cargo-scout">
      <img src="https://travis-ci.com/o0Ignition0o/cargo-scout.svg?branch=master" alt="Travis Build Status">
  </a>
  <a href="https://codecov.io/gh/o0Ignition0o/cargo-scout">
    <img src="https://codecov.io/gh/o0Ignition0o/cargo-scout/branch/master/graph/badge.svg" />
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

A cargo subcommand to get [clippy::pedantic](https://github.com/rust-lang/rust-clippy#clippy) lints or [rustfmt](https://github.com/rust-lang/rustfmt) lints for the changes you have made in a codebase.

Commands and their bash pseudocode would probably look like this:

```bash
# run clippy pedantic on a diff
$ cargo-scout lint # git diff | cargo clippy -- -D clippy::pedantic

# run rustfmt on a diff
$ cargo-scout fmt # git diff | cargo fmt --check
```
There's more to it (the commented code wouldn't work), such as workspace management and quite a lot of flags that will hopefully match your usecase. You can find them by running the commands with -h or --help.

If cargo-scout is missing a feature for you to use it, consider filing an issue!


```bash
$ cargo-scout -h
$ cargo-scout lint -h
$ cargo-scout fmt -h
```

## Current Status

cargo-scout is experimental and in a very rough draft for now.

The current minimum Rust version supported is 1.37 stable.

## Prerequisites
Git: In order to compute a set of changes, it requires a project running git.

The linter uses [clippy](https://github.com/rust-lang/rust-clippy) and the formatter uses [rustfmt](https://github.com/rust-lang/rustfmt). Head over to the respective links to figure out how to install it.

Rust nightly: Some commands require a nightly edition of rust, because the features we use aren't available in stable yet ([rustfmt --emit json](https://github.com/rust-lang/rustfmt/issues/3947) and some [cargo clippy features in a workspace setting](https://github.com/rust-lang/cargo/issues/4942)).

We try to keep a close eye to the relevant tracking issues and hope we can switch it to stable soon. If the issues evolved and we didn't notice, please file an issue and let us know !


## How to install
```bash
$ cargo install cargo-scout
```

## How to run it

Open a shell, go to the project you would like to run the command in, and run cargo-scout, with an optional target branch:
```bash
$ cargo-scout lint # clippy::pedantic lints on a diff with master
$ cargo-scout fmt # rustfmt lints on a diff with master
```

Each command and subcommand supports -h and --help:

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