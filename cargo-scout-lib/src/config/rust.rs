use crate::config::Config;
use std::path::Path;

/// This struct represents a Cargo project configuration.
pub struct CargoConfig {
    members: Vec<String>,
}

impl Config for CargoConfig {
    #[must_use]
    fn members(&self) -> Vec<String> {
        self.members.clone()
    }
}

impl CargoConfig {
    /// This function will instantiate a Config from a Cargo.toml path.
    ///
    /// If in a workspace, `get_members` will return the members
    /// of the [[workspace]] members section in Cargo.toml.
    ///
    /// Else, it will return `vec![".".to_string()]`
    ///
    /// # cargo-scout-lib example
    /// ```
    /// # use cargo_scout_lib::config::Config;
    /// # use cargo_scout_lib::config::rust::CargoConfig;
    /// let config = CargoConfig::from_manifest_path("Cargo.toml")?;
    /// // There is only one directory to lint, which is the current one.
    /// assert_eq!(vec!["."], config.members());
    /// # Ok::<(), cargo_scout_lib::Error>(())
    /// ```
    ///
    /// # cargo-scout workspace example
    /// ```
    /// # use cargo_scout_lib::config::Config;
    /// # use cargo_scout_lib::config::rust::CargoConfig;
    /// let config = CargoConfig::from_manifest_path("../Cargo.toml")?;
    /// // We will lint `./cargo-scout` and `./cargo-scout-lib`.
    /// assert_eq!(vec!["cargo-scout".to_string(), "cargo-scout-lib".to_string()], config.members());
    /// # Ok::<(), cargo_scout_lib::Error>(())
    /// ```
    #[allow(clippy::missing_errors_doc)]
    pub fn from_manifest_path(p: impl AsRef<Path>) -> Result<Self, crate::error::Error> {
        Ok(Self::from_manifest(cargo_toml::Manifest::from_path(p)?))
    }

    fn from_manifest(m: cargo_toml::Manifest) -> Self {
        if let Some(w) = m.workspace {
            Self { members: w.members }
        } else {
            Self {
                // Project root only
                members: vec![".".to_string()],
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::rust::CargoConfig;
    use crate::config::Config;

    #[test]
    fn test_not_workspace_manifest() {
        let manifest = cargo_toml::Manifest::from_path("Cargo.toml").unwrap();
        // Make sure we actually parsed the manifest
        assert_eq!("cargo-scout-lib", manifest.clone().package.unwrap().name);
        let config = CargoConfig::from_manifest(manifest);
        assert_eq!(vec!["."], config.members());
    }
    #[test]
    fn test_not_workspace_path() {
        let config = CargoConfig::from_manifest_path("Cargo.toml").unwrap();
        assert_eq!(vec!["."], config.members());
    }
    #[test]
    fn test_neqo_members_manifest() {
        let neqo_toml = r#"[workspace]
        members = [
          "neqo-client",
          "neqo-common",
          "neqo-crypto",
          "neqo-http3",
          "neqo-http3-server",
          "neqo-qpack",
          "neqo-server",
          "neqo-transport",
          "neqo-interop",
          "test-fixture",
        ]"#;

        let manifest = cargo_toml::Manifest::from_slice(neqo_toml.as_bytes()).unwrap();
        let config = CargoConfig::from_manifest(manifest);
        assert_eq!(
            vec![
                "neqo-client",
                "neqo-common",
                "neqo-crypto",
                "neqo-http3",
                "neqo-http3-server",
                "neqo-qpack",
                "neqo-server",
                "neqo-transport",
                "neqo-interop",
                "test-fixture"
            ],
            config.members()
        );
    }
}
