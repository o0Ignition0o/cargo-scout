use super::{Section, VCS};
use crate::error::Error;
use crate::utils::get_absolute_file_path;
use git2::{Delta, DiffOptions, Repository};
use std::path::Path;

pub struct Git {
    target_branch: String,
}

impl Default for Git {
    #[must_use]
    fn default() -> Self {
        Self {
            target_branch: "HEAD".to_string(),
        }
    }
}

impl Git {
    #[must_use]
    pub fn with_target(target_branch: String) -> Self {
        Self { target_branch }
    }
}

impl VCS for Git {
    fn sections<P>(&self, repo_path: P) -> Result<Vec<Section>, Error>
    where
        P: AsRef<Path>,
    {
        println!("[VCS] - Getting diff with target {}", &self.target_branch);
        let repo = Repository::discover(repo_path)?;
        let tree = repo.revparse_single(&self.target_branch)?.peel_to_tree()?;
        let mut config = DiffOptions::default();
        config
            .context_lines(0)
            .show_untracked_content(true)
            .recurse_untracked_dirs(true);
        let diff = repo.diff_tree_to_workdir_with_index(Some(&tree), Some(&mut config))?;
        let mut sections = Vec::new();
        diff.foreach(
            &mut |_delta, _progress| true,
            None,
            Some(&mut |delta, hunk| {
                match delta.status() {
                    Delta::Modified | Delta::Added | Delta::Untracked => {
                        if let Some(file_path) = delta.new_file().path() {
                            // Path returns the path of the entry relative to the working directory.
                            // We can get the absolute path
                            if let Ok(file_name) = get_absolute_file_path(&file_path) {
                                if file_name.ends_with(".rs") {
                                    sections.push(Section {
                                        file_name,
                                        line_start: hunk.new_start(),
                                        line_end: hunk.new_start() + hunk.new_lines(),
                                    });
                                }
                            }
                        }
                    }
                    _ => {}
                }
                true
            }),
            None,
        )?;
        Ok(sections)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    type Result<T> = std::result::Result<T, Error>;

    #[test]
    fn no_changes() -> Result<()> {
        let repo = RepoFixture::new()?;
        let git = Git::default();
        let sections = git.sections(repo.path())?;
        assert!(sections.is_empty());
        Ok(())
    }

    #[test]
    fn added_files() -> Result<()> {
        let repo = RepoFixture::new()?
            .write("foo.rs", "test_files/git/added/foo.rs")?
            .write("bar.rs", "test_files/git/added/bar.rs")?
            .stage(&["foo.rs", "bar.rs"])?;

        let expected = vec![
            Section {
                file_name: get_absolute_file_path(&"bar.rs")?,
                line_start: 1,
                line_end: 5,
            },
            Section {
                file_name: get_absolute_file_path(&"foo.rs")?,
                line_start: 1,
                line_end: 7,
            },
        ];

        let git = Git::default();
        let actual = git.sections(repo.path())?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn untracked_files() -> Result<()> {
        let repo = RepoFixture::new()?
            .write("foo.rs", "test_files/git/added/foo.rs")?
            .write("inside/some/dir/bar.rs", "test_files/git/added/bar.rs")?;

        let expected = vec![
            Section {
                file_name: get_absolute_file_path(&"foo.rs")?,
                line_start: 1,
                line_end: 7,
            },
            Section {
                file_name: get_absolute_file_path(&"inside/some/dir/bar.rs")?,
                line_start: 1,
                line_end: 5,
            },
        ];

        let git = Git::default();
        let actual = git.sections(repo.path())?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn modified_files() -> Result<()> {
        let files = &["foo.rs", "bar.rs"];
        let repo = RepoFixture::new()?
            .write("foo.rs", "test_files/git/modified/old/foo.rs")?
            .write("bar.rs", "test_files/git/modified/old/bar.rs")?
            .stage(files)?
            .commit("master", files)?
            .write("foo.rs", "test_files/git/modified/new/foo.rs")?
            .write("bar.rs", "test_files/git/modified/new/bar.rs")?;

        let expected = vec![
            Section {
                file_name: get_absolute_file_path(&"bar.rs")?,
                line_start: 1,
                line_end: 2,
            },
            Section {
                file_name: get_absolute_file_path(&"bar.rs")?,
                line_start: 5,
                line_end: 9,
            },
            Section {
                file_name: get_absolute_file_path(&"foo.rs")?,
                line_start: 3,
                line_end: 4,
            },
            Section {
                file_name: get_absolute_file_path(&"foo.rs")?,
                line_start: 6,
                line_end: 7,
            },
        ];

        let git = Git::default();
        let actual = git.sections(repo.path())?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn mixed_extensions() -> Result<()> {
        let repo = RepoFixture::new()?
            .write("foo.rs", "test_files/git/mixed/foo.rs")?
            .write("bar.txt", "test_files/git/mixed/bar.txt")?
            .stage(&["foo.rs", "bar.txt"])?;

        let expected = vec![Section {
            file_name: get_absolute_file_path(&"foo.rs")?,
            line_start: 1,
            line_end: 7,
        }];

        let git = Git::default();
        let actual = git.sections(repo.path())?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn other_branch() -> Result<()> {
        let repo = RepoFixture::new()?
            .branch("other")?
            .write("foo.rs", "test_files/git/modified/old/foo.rs")?
            .stage(&["foo.rs"])?
            .commit("other", &["foo.rs"])?
            .write("foo.rs", "test_files/git/modified/new/foo.rs")?;

        let expected = vec![
            Section {
                file_name: get_absolute_file_path(&"foo.rs")?,
                line_start: 3,
                line_end: 4,
            },
            Section {
                file_name: get_absolute_file_path(&"foo.rs")?,
                line_start: 6,
                line_end: 7,
            },
        ];

        let git = Git::with_target("other".to_string());
        let actual = git.sections(repo.path())?;
        assert_eq!(expected, actual);
        Ok(())
    }

    struct RepoFixture {
        dir: TempDir,
        repo: Repository,
    }

    impl RepoFixture {
        pub fn new() -> Result<Self> {
            let dir = TempDir::new()?;
            let repo = Repository::init(dir.path())?;
            {
                // Set mandatory configuration
                let mut config = repo.config()?;
                config.set_str("user.name", "name")?;
                config.set_str("user.email", "email")?;

                // Write initial commit
                let id = repo.index()?.write_tree()?;
                let tree = repo.find_tree(id)?;
                let sig = repo.signature()?;
                repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])?;
            }
            Ok(Self { dir, repo })
        }

        pub fn write<P: AsRef<Path>>(self, path: P, test_file: &str) -> Result<Self> {
            let parent_dir = path.as_ref().parent().unwrap();
            fs::create_dir_all(self.dir.path().join(parent_dir))?;

            let contents = fs::read_to_string(test_file)?;
            let mut file = File::create(self.dir.path().join(path))?;
            file.write_all(contents.as_bytes())?;
            Ok(self)
        }

        pub fn stage(self, paths: &[&str]) -> Result<Self> {
            let mut index = self.repo.index()?;
            for path in paths {
                index.add_path(path.as_ref())?;
            }
            index.write()?;
            Ok(self)
        }

        pub fn commit(self, branch: &str, paths: &[&str]) -> Result<Self> {
            {
                let mut index = self.repo.index()?;
                for path in paths {
                    index.add_path(path.as_ref())?;
                }

                let id = index.write_tree()?;
                let tree = self.repo.find_tree(id)?;
                let sig = self.repo.signature()?;

                let target = self.repo.head()?.target().unwrap();
                let parent = self.repo.find_commit(target)?;

                let name = format!("refs/heads/{}", branch);
                self.repo
                    .commit(Some(&name), &sig, &sig, "some commit", &tree, &[&parent])?;
            }
            Ok(self)
        }

        pub fn branch(self, name: &str) -> Result<Self> {
            {
                let target = self.repo.head()?.target().unwrap();
                let parent = self.repo.find_commit(target)?;

                self.repo.branch(name, &parent, false)?;
            }
            Ok(self)
        }

        pub fn path(&self) -> &Path {
            self.dir.path()
        }
    }
}
