use serde::Deserialize;

#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct Parser {
    members: Vec<String>,
}

impl Parser {
    pub fn from_manifest_path(p: impl AsRef<std::path::Path>) -> Result<Self, crate::error::Error> {
        Ok(Self::from_manifest(cargo_toml::Manifest::from_path(p)?))
    }
    pub fn is_workspace(&self) -> bool {
        !self.members.is_empty()
    }

    fn new() -> Self {
        Self { members: vec![] }
    }

    fn from_manifest(m: cargo_toml::Manifest) -> Self {
        if let Some(w) = m.workspace {
            Self { members: w.members }
        } else {
            Self::new()
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_not_workspace_manifest() {
        use crate::cargo::Parser;
        let no_members: Vec<String> = Vec::new();
        let manifest = cargo_toml::Manifest::from_path("Cargo.toml").unwrap();
        // Make sure we actually parsed the manifest
        assert_eq!("cargo-scout", manifest.clone().package.unwrap().name);
        let parser = Parser::from_manifest(manifest);
        assert!(!parser.is_workspace());
        assert_eq!(no_members, parser.members);
    }
    #[test]
    fn test_not_workspace_path() {
        use crate::cargo::Parser;
        let no_members: Vec<String> = Vec::new();
        let parser = Parser::from_manifest_path("Cargo.toml").unwrap();
        assert!(!parser.is_workspace());
        assert_eq!(no_members, parser.members);
    }
    #[test]
    fn test_neqo_members_manifest() {
        use crate::cargo::Parser;
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

        let parser = Parser::from_manifest(manifest);

        assert!(parser.is_workspace());
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
            parser.members
        );
    }
}
