pub mod rust;

/// This trait is responsible for providing a list of members,
/// which are directories to be linted against.
pub trait Config {
    /// This function should return a list of relative paths
    /// a linter will iterate on.
    ///
    /// If only the current working directory must be checked, it must return `vec![".".to_string()]`
    ///
    /// If several directories must be checked,
    /// return their relative path as strings.
    ///
    /// For example if your current working directory is `foo`,
    /// and you want to check `./bar` and `./baz`,
    /// return `vec!["bar".to_string(), "baz".to_string()]`
    ///
    /// # Example with the root directory
    /// ```
    /// # use cargo_scout_lib::config::Config;
    /// # struct CustomConfig{}
    /// # impl CustomConfig {
    /// #    fn new() -> Self {
    /// #        Self {}
    /// #    }
    /// # }
    /// # // Your own implementation goes here
    /// # impl Config for CustomConfig {
    /// #    fn get_members(&self) -> Vec<String> {
    /// #        vec![".".to_string()]
    /// #    }
    /// # }
    /// let config = CustomConfig::new();
    /// // Only the current directory must be linted
    /// assert_eq!(vec![".".to_string()], config.get_members());
    /// ```
    ///
    /// # Example with two subdirectories
    /// ```
    /// # use cargo_scout_lib::config::Config;
    /// # struct CustomConfig{}    
    /// # impl CustomConfig {
    /// #    fn new() -> Self {
    /// #        Self {}
    /// #    }
    /// # }
    /// # // Your own implementation goes here
    /// # impl Config for CustomConfig {
    /// #    fn get_members(&self) -> Vec<String> {
    /// #        vec!["foo".to_string(), "bar".to_string()]
    /// #    }
    /// # }
    /// let config = CustomConfig::new();
    /// // Directories ./foo and ./bar must be linted
    /// assert_eq!(vec!["foo".to_string(), "bar".to_string()], config.get_members());
    /// ```
    ///
    /// # Implementing your own Config
    /// ```
    /// use cargo_scout_lib::config::Config;
    ///
    /// struct CustomConfig{}
    ///
    /// # impl CustomConfig {
    /// #   fn new() -> Self {
    /// #       Self {}
    /// #   }
    /// # }
    /// impl Config for CustomConfig {
    ///    fn get_members(&self) -> Vec<String> {
    ///        // Your own code to fetch the list of
    ///        // directories to iterate on goes here
    ///        # vec![".".to_string()]
    ///    }
    /// }
    /// ```
    fn get_members(&self) -> Vec<String>;
}
