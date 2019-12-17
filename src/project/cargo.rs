use crate::project::Config as ProjectConfig;
use serde::Deserialize;

#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct Config {
    no_default_features: bool,
    all_features: bool,
    members: Vec<String>,
}

impl ProjectConfig for Config {
    fn linter_must_iterate(&self) -> bool {
        !self.members.is_empty() && (self.no_default_features || self.all_features)
    }
    fn get_members(&self) -> Vec<String> {
        self.members.clone()
    }
}

impl Config {
    pub fn from_manifest_path(p: impl AsRef<std::path::Path>) -> Result<Self, crate::error::Error> {
        Ok(Self::from_manifest(cargo_toml::Manifest::from_path(p)?))
    }

    pub fn set_no_default_features(&mut self, no_default_features: bool) -> &mut Self {
        self.no_default_features = no_default_features;
        self
    }

    pub fn set_all_features(&mut self, all_features: bool) -> &mut Self {
        self.all_features = all_features;
        self
    }

    fn from_manifest(m: cargo_toml::Manifest) -> Self {
        if let Some(w) = m.workspace {
            Self {
                members: w.members,
                no_default_features: false,
                all_features: false,
            }
        } else {
            Self {
                members: Vec::new(),
                no_default_features: false,
                all_features: false,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::project::cargo::Config;
    use crate::project::Config as ProjectConfig;

    #[test]
    fn test_not_workspace_manifest() {
        let no_members: Vec<String> = Vec::new();
        let manifest = cargo_toml::Manifest::from_path("Cargo.toml").unwrap();
        // Make sure we actually parsed the manifest
        assert_eq!("cargo-scout", manifest.clone().package.unwrap().name);
        let mut project = Config::from_manifest(manifest);
        assert!(!project.linter_must_iterate());
        assert_eq!(no_members, project.get_members());

        // Config must not iterate if not running in a workspace,
        // regardless of the passed flags
        project.set_all_features(true);
        assert!(!project.linter_must_iterate());
        project.set_no_default_features(true);
        assert!(!project.linter_must_iterate());
        project.set_all_features(false);
        assert!(!project.linter_must_iterate());
    }
    #[test]
    fn test_not_workspace_path() {
        let no_members: Vec<String> = Vec::new();
        let mut project = Config::from_manifest_path("Cargo.toml").unwrap();
        assert!(!project.linter_must_iterate());
        assert_eq!(no_members, project.get_members());

        // Config must not iterate if not running in a workspace,
        // regardless of the passed flags
        project.set_all_features(true);
        assert!(!project.linter_must_iterate());
        project.set_no_default_features(true);
        assert!(!project.linter_must_iterate());
        project.set_all_features(false);
        assert!(!project.linter_must_iterate());
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

        let mut project = Config::from_manifest(manifest);

        assert!(!project.linter_must_iterate());
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
            project.get_members()
        );
        // Config must iterate if running in a workspace
        // With all features or no default features is enabled
        project.set_all_features(true);
        assert!(project.linter_must_iterate());
        project.set_no_default_features(true);
        assert!(project.linter_must_iterate());
        project.set_all_features(false);
        assert!(project.linter_must_iterate());
    }
}
