use crate::clippy;
use crate::git;

// Check if clippy_lint and git_section have overlapped lines
fn lines_in_range(clippy_lint: &clippy::Span, git_section: &git::Section) -> bool {
    // If git_section.line_start is included in the clippy_lint span
    clippy_lint.line_start <= git_section.line_start && git_section.line_start <= clippy_lint.line_end ||
    // If clippy_lint.line_start is included in the git_section span
    git_section.line_start <= clippy_lint.line_start && clippy_lint.line_start <= git_section.line_end
}

fn files_match(clippy_lint: &clippy::Span, git_section: &git::Section) -> bool {
    // Git diff paths and clippy paths don't get along too well on Windows...
    clippy_lint.file_name.replace("\\", "/") == git_section.file_name.replace("\\", "/")
}

pub fn get_lints_from_diff(
    lints: &[clippy::Lint],
    diffs: &[git::Section],
    _verbose: bool,
) -> Vec<clippy::Lint> {
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
mod intersections_tests {
    use crate::clippy::Span;
    use crate::git::Section;
    use crate::intersections::files_match;

    type TestSection = (&'static str, i32, i32);
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

    fn in_range(lint: (&str, i32, i32), section: (&str, i32, i32)) -> bool {
        use crate::intersections::lines_in_range;
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
