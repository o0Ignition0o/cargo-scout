#[derive(Debug)]
pub enum Error {
    ScoutBuilder,
    CargoToml(cargo_toml::Error),
    Command(String),
    Utf8(std::string::FromUtf8Error),
    Json(serde_json::Error),
    NotClean,
    Io(std::io::Error),
    Git(git2::Error),
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Self::Utf8(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Self::Command(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<cargo_toml::Error> for Error {
    fn from(err: cargo_toml::Error) -> Self {
        Self::CargoToml(err)
    }
}

impl From<git2::Error> for Error {
    fn from(err: git2::Error) -> Self {
        Self::Git(err)
    }
}
