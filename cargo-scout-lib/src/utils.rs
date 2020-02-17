use crate::error::Error;
use std::path::Path;

pub fn get_absolute_file_path(file_path: impl AsRef<Path>) -> Result<String, Error> {
    let mut absolute_path = std::env::current_dir()?;
    absolute_path.push(file_path);
    Ok(absolute_path.to_string_lossy().to_string())
}

pub fn get_relative_file_path(file_path: impl AsRef<Path>) -> Result<String, Error> {
    let current_dir = std::env::current_dir()?;
    let relative_path = file_path
        .as_ref()
        .strip_prefix(current_dir.to_string_lossy().to_string())?;
    Ok(relative_path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_file_path_conversion() -> Result<(), crate::error::Error> {
        let relative_path = "foo/bar.rs";
        assert_eq!(
            relative_path,
            get_relative_file_path(get_absolute_file_path(relative_path)?)?
        );
        Ok(())
    }
}
