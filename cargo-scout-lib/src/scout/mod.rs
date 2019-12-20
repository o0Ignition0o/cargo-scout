use crate::config::*;
use crate::linter::*;
use crate::vcs::*;

pub struct Scout<V, C, L>
where
    V: VCS,
    C: Config,
    L: Linter,
{
    vcs: V,
    config: C,
    linter: L,
}

impl<V, C, L> Scout<V, C, L>
where
    V: VCS,
    C: Config,
    L: Linter,
{
    pub fn new(vcs: V, config: C, linter: L) -> Self {
        Self {
            vcs,
            config,
            linter,
        }
    }
    pub fn run(&self) -> Result<Vec<Lint>, crate::error::Error> {
        let diff_sections = self.vcs.get_sections(".")?;
        let current_dir = std::fs::canonicalize(".")?;
        let mut lints = Vec::new();
        let members = self.config.get_members();
        // There's no need to run the linter on members where no changes have been made
        let relevant_members = members.iter().filter(|m| diff_in_member(m, &diff_sections));
        for m in relevant_members {
            lints.extend(self.linter.get_lints(current_dir.join(m))?);
        }
        println!("[Scout] - checking for intersections");
        Ok(get_lints_from_diff(&lints, &diff_sections))
    }
}

fn diff_in_member(member: &str, sections: &[Section]) -> bool {
    for s in sections {
        if s.file_name.starts_with(&member) {
            return true;
        }
    }
    false
}

// Check if clippy_lint and git_section have overlapped lines
fn lines_in_range(clippy_lint: &Span, git_section: &Section) -> bool {
    // If git_section.line_start is included in the clippy_lint span
    clippy_lint.line_start <= git_section.line_start && git_section.line_start <= clippy_lint.line_end ||
    // If clippy_lint.line_start is included in the git_section span
    git_section.line_start <= clippy_lint.line_start && clippy_lint.line_start <= git_section.line_end
}

fn files_match(clippy_lint: &Span, git_section: &Section) -> bool {
    // Git diff paths and clippy paths don't get along too well on Windows...
    clippy_lint.file_name.replace("\\", "/") == git_section.file_name.replace("\\", "/")
}

fn get_lints_from_diff(lints: &[Lint], diffs: &[Section]) -> Vec<Lint> {
    let mut lints_in_diff = Vec::new();
    for diff in diffs {
        let diff_lints = lints.iter().filter(|lint| {
            if let Some(m) = &lint.message {
                for s in &m.spans {
                    if files_match(&s, &diff) && lines_in_range(&s, &diff) {
                        return true;
                    };
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

#[cfg(test)]
mod scout_tests {
    use super::*;
    use crate::config::Config;
    use crate::error::Error;
    use crate::linter::{Lint, Linter};
    use std::cell::RefCell;
    use std::clone::Clone;
    use std::path::{Path, PathBuf};
    use std::rc::Rc;
    struct TestVCS {
        sections: Vec<Section>,
        get_sections_called: RefCell<bool>,
    }
    impl TestVCS {
        pub fn new(sections: Vec<Section>) -> Self {
            Self {
                sections,
                get_sections_called: RefCell::new(false),
            }
        }
    }
    impl VCS for TestVCS {
        fn get_sections<P: AsRef<Path>>(&self, _: P) -> Result<Vec<Section>, Error> {
            *self.get_sections_called.borrow_mut() = true;
            Ok(self.sections.clone())
        }
    }
    struct TestLinter {
        // Using a RefCell here because get_lints
        // takes &self and not &mut self.
        // We use usize here because we will compare it to a Vec::len()
        lints_times_called: Rc<RefCell<usize>>,
    }
    impl TestLinter {
        pub fn new() -> Self {
            Self {
                lints_times_called: Rc::new(RefCell::new(0)),
            }
        }
    }
    impl Linter for TestLinter {
        fn get_lints(&self, _working_dir: PathBuf) -> Result<Vec<Lint>, crate::error::Error> {
            *self.lints_times_called.borrow_mut() += 1;
            Ok(Vec::new())
        }
    }
    struct TestConfig {
        members: Vec<String>,
    }
    impl TestConfig {
        pub fn new(members: Vec<String>) -> Self {
            TestConfig { members }
        }
    }
    impl Config for TestConfig {
        fn get_members(&self) -> Vec<String> {
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
        let actual_times_called = Rc::clone(&linter.lints_times_called);
        let scout = Scout::new(vcs, config, linter);
        // We don't check for the lints result here.
        // It is already tested in the linter tests
        // and in intersection tests
        let _ = scout.run()?;
        assert_eq!(expected_times_called, *actual_times_called.borrow());
        Ok(())
    }

    #[test]
    fn test_scout_no_workspace_one_diff() -> Result<(), crate::error::Error> {
        let diff = vec![Section {
            file_name: "foo/bar.rs".to_string(),
            line_start: 0,
            line_end: 10,
        }];
        let linter = TestLinter::new();
        let vcs = TestVCS::new(diff);
        // The member matches the file name
        let config = TestConfig::new(vec!["foo".to_string()]);
        let expected_times_called = 1;
        let actual_times_called = Rc::clone(&linter.lints_times_called);
        let scout = Scout::new(vcs, config, linter);
        // We don't check for the lints result here.
        // It is already tested in the linter tests
        // and in intersection tests
        let _ = scout.run()?;
        assert_eq!(expected_times_called, *actual_times_called.borrow());
        Ok(())
    }

    #[test]
    fn test_scout_no_workspace_one_diff_not_relevant_member() -> Result<(), crate::error::Error> {
        let diff = vec![Section {
            file_name: "baz/bar.rs".to_string(),
            line_start: 0,
            line_end: 10,
        }];
        let linter = TestLinter::new();
        let vcs = TestVCS::new(diff);
        // The member does not match the file name
        let config = TestConfig::new(vec!["foo".to_string()]);
        let expected_times_called = 0;
        let actual_times_called = Rc::clone(&linter.lints_times_called);
        let scout = Scout::new(vcs, config, linter);
        // We don't check for the lints result here.
        // It is already tested in the linter tests
        // and in intersection tests
        let _ = scout.run()?;
        assert_eq!(expected_times_called, *actual_times_called.borrow());
        Ok(())
    }

    #[test]
    fn test_scout_in_workspace() -> Result<(), crate::error::Error> {
        let diff = vec![
            Section {
                file_name: "member1/bar.rs".to_string(),
                line_start: 0,
                line_end: 10,
            },
            Section {
                file_name: "member2/baz.rs".to_string(),
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
        let actual_times_called = Rc::clone(&linter.lints_times_called);
        let scout = Scout::new(vcs, config, linter);
        // We don't check for the lints result here.
        // It is already tested in the linter tests
        // and in intersection tests
        let _ = scout.run()?;

        assert_eq!(expected_times_called, *actual_times_called.borrow());
        Ok(())
    }
}

#[cfg(test)]
mod intersections_tests {
    use crate::linter::Span;
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
            let lint = range.0;
            let section = range.1;
            let clippy_lint = Span {
                file_name: String::from(lint.0),
                line_start: lint.1,
                line_end: lint.2,
            };
            let git_section = Section {
                file_name: String::from(section.0),
                line_start: section.1,
                line_end: section.2,
            };
            assert!(
                files_match(&clippy_lint, &git_section),
                print!("Expected files match for {} and {}", lint.0, section.0)
            );
        }
    }

    fn assert_no_files_match(ranges: Vec<(TestSection, TestSection)>) {
        use crate::scout::files_match;
        for range in ranges {
            let lint = range.0;
            let section = range.1;
            let clippy_lint = Span {
                file_name: String::from(lint.0),
                line_start: lint.1,
                line_end: lint.2,
            };
            let git_section = Section {
                file_name: String::from(section.0),
                line_start: section.1,
                line_end: section.2,
            };
            assert!(
                !files_match(&clippy_lint, &git_section),
                print!("Expected files match for {} and {}", lint.0, section.0)
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

    fn in_range(lint: (&str, u32, u32), section: (&str, u32, u32)) -> bool {
        use crate::scout::lines_in_range;
        let clippy_lint = Span {
            file_name: String::from(lint.0),
            line_start: lint.1,
            line_end: lint.2,
        };

        let git_section = Section {
            file_name: String::from(section.0),
            line_start: section.1,
            line_end: section.2,
        };
        lines_in_range(&clippy_lint, &git_section)
    }
}
