pub mod git;
use crate::error::Error;
use std::path::Path;

pub trait VCS {
    #[allow(clippy::missing_errors_doc)]
    fn sections<P>(&self, repo_path: P) -> Result<Vec<Section>, Error>
    where
        P: AsRef<Path>;
}

#[derive(Debug, PartialEq, Clone)]
pub struct Section {
    pub file_name: String,
    pub line_start: u32,
    pub line_end: u32,
}
