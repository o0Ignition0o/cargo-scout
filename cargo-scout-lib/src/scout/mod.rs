use crate::config::*;
use crate::healer::Healer;
use crate::linter::{Lint, Linter};
use crate::vcs::*;
use std::path::PathBuf;

pub struct Scout<'s, V, C, L>
where
    V: VCS,
    C: Config,
    L: Linter,
{
    vcs: V,
    config: C,
    linter: &'s L,
}

impl<'s, V, C, L> Scout<'s, V, C, L>
where
    V: VCS,
    C: Config,
    L: Linter,
{
    pub fn new(vcs: V, config: C, linter: &'s L) -> Self {
        Self {
            vcs,
            config,
            linter,
        }
    }
    pub fn run(&self) -> Result<Vec<Lint>, crate::error::Error> {
        let current_dir = std::fs::canonicalize(std::env::current_dir()?)?;
        let diff_sections = self.vcs.sections(current_dir.clone())?;
        let mut lints = Vec::new();
        let config_members = self.config.members();
        let members = config_members.iter().map(|m| {
            let mut member = current_dir.clone();
            member.push(m);
            member
        });
        // There's no need to run the linter on members where no changes have been made
        let relevant_members = members.filter(|m| diff_in_member(m, &diff_sections));
        for m in relevant_members {
            lints.extend(self.linter.lints(current_dir.clone().join(m))?);
        }
        println!("[Scout] - checking for intersections");
        Ok(lints_from_diff(&lints, &diff_sections))
    }
}

pub struct Fixer<H>
where
    H: Healer,
{
    medic: H,
}

impl<H> Fixer<H>
where
    H: Healer,
{
    pub fn new(medic: H) -> Self {
        Self { medic }
    }
    pub fn run(&self, lints: Vec<Lint>) -> Result<(), crate::error::Error> {
        println!("[Scout] - applying fixes");
        self.medic.heal(lints)
    }
}

fn diff_in_member(member: &PathBuf, sections: &[Section]) -> bool {
    if let Some(m) = member.to_str() {
        for s in sections {
            if s.file_name.starts_with(&m) {
                return true;
            }
        }
    }
    false
}

// Check if lint and git_section have overlapped lines
fn lines_in_range(lint: &Lint, git_section: &Section) -> bool {
    // If git_section.line_start is included in the lint span
    lint.location.lines[0] <= git_section.line_start && git_section.line_start <= lint.location.lines[1] ||
    // If lint.line_start is included in the git_section span
    git_section.line_start <= lint.location.lines[0] && lint.location.lines[0] <= git_section.line_end
}

fn files_match(lint: &Lint, git_section: &Section) -> bool {
    // Git diff paths and clippy paths don't get along too well on Windows...
    lint.location.path.replace("\\", "/") == git_section.file_name.replace("\\", "/")
}

fn lints_from_diff(lints: &[Lint], diffs: &[Section]) -> Vec<Lint> {
    let mut lints_in_diff = Vec::new();
    for diff in diffs {
        let diff_lints = lints
            .iter()
            .filter(|lint| files_match(&lint, &diff) && lines_in_range(&lint, &diff));
        for l in diff_lints {
            lints_in_diff.push(l.clone());
        }
    }
    lints_in_diff
}

#[cfg(test)]
mod scout_tests {
    use super::*;
    use crate::config::Config;
    use crate::error::Error;
    use crate::linter::{Lint, Linter, Location};
    use crate::utils::get_absolute_file_path;
    use std::cell::RefCell;
    use std::clone::Clone;
    use std::path::{Path, PathBuf};
    struct TestVCS {
        sections: Vec<Section>,
        sections_called: RefCell<bool>,
    }
    impl TestVCS {
        pub fn new(sections: Vec<Section>) -> Self {
            Self {
                sections,
                sections_called: RefCell::new(false),
            }
        }
    }
    impl VCS for TestVCS {
        fn sections<P: AsRef<Path>>(&self, _: P) -> Result<Vec<Section>, Error> {
            *self.sections_called.borrow_mut() = true;
            Ok(self.sections.clone())
        }
    }
    struct TestLinter {
        // Using a RefCell here because lints
        // takes &self and not &mut self.
        // We use usize here because we will compare it to a Vec::len()
        times_called: RefCell<usize>,
        lints: Vec<Lint>,
    }
    impl TestLinter {
        pub fn new() -> Self {
            Self {
                times_called: RefCell::new(0),
                lints: Vec::new(),
            }
        }

        pub fn with_lints(lints: Vec<Lint>) -> Self {
            Self {
                times_called: RefCell::new(0),
                lints,
            }
        }
    }
    impl Linter for TestLinter {
        fn lints(
            &self,
            _working_dir: impl Into<PathBuf>,
        ) -> Result<Vec<Lint>, crate::error::Error> {
            *self.times_called.borrow_mut() += 1;
            Ok(self.lints.clone())
        }
    }
    impl Healer for TestLinter {
        fn heal(&self, _lints: Vec<Lint>) -> Result<(), crate::error::Error> {
            *self.times_called.borrow_mut() += 1;
            Ok(())
        }
    }
    struct TestConfig {
        members: Vec<String>,
    }
    impl TestConfig {
        pub fn new(members: Vec<String>) -> Self {
            Self { members }
        }
    }
    impl Config for TestConfig {
        fn members(&self) -> Vec<String> {
            self.members.clone()
        }
    }

    #[test]
    fn test_scout_no_workspace_no_diff() -> Result<(), crate::error::Error> {
        let linter = TestLinter::new();
        let vcs = TestVCS::new(Vec::new());
        // No members so we won't have to iterate
        let config = TestConfig::new(Vec::new());
        let expected_times_called = 0;
        let scout = Scout::new(vcs, config, &linter);
        // We don't check for the lints result here.
        // It is already tested in the linter tests
        // and in intersection tests
        let _ = scout.run()?;
        assert_eq!(expected_times_called, *linter.times_called.borrow());
        Ok(())
    }

    #[test]
    fn test_scout_no_workspace_one_diff() -> Result<(), crate::error::Error> {
        let diff = vec![Section {
            file_name: get_absolute_file_path("foo/bar.rs")?,
            line_start: 0,
            line_end: 10,
        }];

        let lints = vec![
            Lint {
                location: Location {
                    lines: [2, 2],
                    path: get_absolute_file_path("foo/bar.rs")?,
                },
                message: "Test lint".to_string(),
            },
            Lint {
                location: Location {
                    lines: [12, 22],
                    path: get_absolute_file_path("foo/bar.rs")?,
                },
                message: "This lint is not in diff".to_string(),
            },
        ];

        let expected_lints_from_diff = vec![Lint {
            location: Location {
                lines: [2, 2],
                path: get_absolute_file_path("foo/bar.rs")?,
            },
            message: "Test lint".to_string(),
        }];

        let linter = TestLinter::with_lints(lints);
        let vcs = TestVCS::new(diff);
        // The member matches the file name
        let config = TestConfig::new(vec!["foo".to_string()]);
        let expected_times_called = 1;
        let scout = Scout::new(vcs, config, &linter);
        // We don't check for the lints result here.
        // It is already tested in the linter tests
        // and in intersection tests
        let actual_lints_from_diff = scout.run()?;
        assert_eq!(expected_times_called, *linter.times_called.borrow());
        assert_eq!(expected_lints_from_diff, actual_lints_from_diff);
        Ok(())
    }

    #[test]
    fn test_scout_no_workspace_one_diff_not_relevant_member() -> Result<(), crate::error::Error> {
        let diff = vec![Section {
            file_name: get_absolute_file_path("baz/bar.rs")?,
            line_start: 0,
            line_end: 10,
        }];
        let linter = TestLinter::new();
        let vcs = TestVCS::new(diff);
        // The member does not match the file name
        let config = TestConfig::new(vec!["foo".to_string()]);
        let expected_times_called = 0;
        let scout = Scout::new(vcs, config, &linter);
        // We don't check for the lints result here.
        // It is already tested in the linter tests
        // and in intersection tests
        let _ = scout.run()?;
        assert_eq!(expected_times_called, *linter.times_called.borrow());
        Ok(())
    }

    #[test]
    fn test_scout_in_workspace() -> Result<(), crate::error::Error> {
        let diff = vec![
            Section {
                file_name: get_absolute_file_path("member1/bar.rs")?,
                line_start: 0,
                line_end: 10,
            },
            Section {
                file_name: get_absolute_file_path("member2/bar.rs")?,
                line_start: 0,
                line_end: 10,
            },
        ];
        let linter = TestLinter::new();
        let vcs = TestVCS::new(diff);
        // The config has members, we will iterate
        let config = TestConfig::new(vec![
            "member1".to_string(),
            "member2".to_string(),
            "member3".to_string(),
        ]);
        // We should run the linter on member1 and member2
        let expected_times_called = 2;
        let scout = Scout::new(vcs, config, &linter);
        // We don't check for the lints result here.
        // It is already tested in the linter tests
        // and in intersection tests
        let _ = scout.run()?;

        assert_eq!(expected_times_called, *linter.times_called.borrow());
        Ok(())
    }

    #[test]
    fn test_heal() -> Result<(), crate::error::Error> {
        let fixer = TestLinter::new();
        let config = TestConfig::new(Vec::new());
        let vcs = TestVCS::new(Vec::new());

        let expected_times_called = 1;
        let scout = Scout::new(vcs, config, &fixer);
        let lints = scout.run()?;
        fixer.heal(lints)?;
        assert_eq!(expected_times_called, *fixer.times_called.borrow());
        Ok(())
    }
}

#[cfg(test)]
mod intersections_tests {
    use crate::linter::{Lint, Location};
    use crate::vcs::Section;

    type TestSection = (&'static str, u32, u32);
    #[test]

    fn test_files_match() {
        let files_to_test = vec![
            (("foo.rs", 1, 10), ("foo.rs", 5, 12)),
            (("bar.rs", 1, 10), ("bar.rs", 5, 12)),
            (("foo/bar/baz.rs", 1, 10), ("foo/bar/baz.rs", 5, 12)),
            (("foo\\bar\\baz.rs", 1, 10), ("foo/bar/baz.rs", 9, 12)),
            (("foo/1.rs", 1, 10), ("foo/1.rs", 5, 12)),
        ];
        assert_all_files_match(files_to_test);
    }

    #[test]
    fn test_files_dont_match() {
        let files_to_test = vec![
            (("foo.rs", 1, 10), ("foo1.rs", 5, 12)),
            (("bar.rs", 1, 10), ("baz.rs", 5, 12)),
            (("bar.rs", 1, 10), ("bar.js", 5, 12)),
            (("foo/bar/baz.rs", 1, 10), ("/foo/bar/baz.rs", 5, 12)),
            (("foo\\\\bar\\baz.rs", 1, 10), ("foo/bar/baz.rs", 9, 12)),
            (("foo/1.rs", 1, 10), ("foo/2.rs", 5, 12)),
        ];
        assert_no_files_match(files_to_test);
    }

    #[test]
    fn test_lines_in_range_simple() {
        let ranges_to_test = vec![
            (("foo.rs", 1, 10), ("foo.rs", 5, 12)),
            (("foo.rs", 1, 10), ("foo.rs", 5, 11)),
            (("foo.rs", 1, 10), ("foo.rs", 10, 19)),
            (("foo.rs", 1, 10), ("foo.rs", 9, 12)),
            (("foo.rs", 8, 16), ("foo.rs", 5, 12)),
        ];
        assert_all_in_range(ranges_to_test);
    }

    #[test]
    fn test_lines_not_in_range_simple() {
        let ranges_to_test = vec![
            (("foo.rs", 1, 10), ("foo.rs", 11, 12)),
            (("foo.rs", 2, 10), ("foo.rs", 0, 1)),
            (("foo.rs", 15, 20), ("foo.rs", 21, 30)),
            (("foo.rs", 15, 20), ("foo.rs", 10, 14)),
            (("foo.rs", 1, 1), ("foo.rs", 2, 2)),
        ];
        assert_all_not_in_range(ranges_to_test);
    }

    fn assert_all_files_match(ranges: Vec<(TestSection, TestSection)>) {
        use crate::scout::files_match;
        for range in ranges {
            let lint_section = range.0;
            let git_section = range.1;
            let lint = Lint {
                message: String::new(),
                location: Location {
                    path: String::from(lint_section.0),
                    lines: [lint_section.1, lint_section.2],
                },
            };
            let git = Section {
                file_name: String::from(git_section.0),
                line_start: git_section.1,
                line_end: git_section.2,
            };
            assert!(
                files_match(&lint, &git),
                print!(
                    "Expected files match for {} and {}",
                    lint_section.0, git_section.0
                )
            );
        }
    }

    fn assert_no_files_match(ranges: Vec<(TestSection, TestSection)>) {
        use crate::scout::files_match;
        for range in ranges {
            let lint_section = range.0;
            let git_section = range.1;
            let lint = Lint {
                message: String::new(),
                location: Location {
                    path: String::from(lint_section.0),
                    lines: [lint_section.1, lint_section.2],
                },
            };
            let git = Section {
                file_name: String::from(git_section.0),
                line_start: git_section.1,
                line_end: git_section.2,
            };
            assert!(
                !files_match(&lint, &git),
                print!(
                    "Expected files not to match for {} and {}",
                    lint_section.0, git_section.0
                )
            );
        }
    }

    fn assert_all_in_range(ranges: Vec<(TestSection, TestSection)>) {
        for range in ranges {
            let lint = range.0;
            let section = range.1;
            assert!(
                in_range(lint, section),
                print!(
                    "Expected in range, found not in range for \n {:#?} and {:#?}",
                    lint, section
                )
            );
        }
    }

    fn assert_all_not_in_range(ranges: Vec<(TestSection, TestSection)>) {
        for range in ranges {
            let lint = range.0;
            let section = range.1;
            assert!(
                !in_range(lint, section),
                print!(
                    "Expected not in range, found in range for \n {:#?} and {:#?}",
                    lint, section
                )
            );
        }
    }

    fn in_range(lint_section: (&str, u32, u32), git_section: (&str, u32, u32)) -> bool {
        use crate::scout::lines_in_range;
        let lint = Lint {
            message: String::new(),
            location: Location {
                path: String::from(lint_section.0),
                lines: [lint_section.1, lint_section.2],
            },
        };

        let git_section = Section {
            file_name: String::from(git_section.0),
            line_start: git_section.1,
            line_end: git_section.2,
        };
        lines_in_range(&lint, &git_section)
    }
}
