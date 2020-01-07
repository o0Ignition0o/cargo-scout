use crate::error::Error;
use std::path::Path;

pub fn get_absolute_file_path(file_path: impl AsRef<Path>) -> Result<String, Error> {
    let mut absolute_path = std::env::current_dir()?;
    absolute_path.push(file_path);
    Ok(absolute_path.to_string_lossy().to_string())
}
