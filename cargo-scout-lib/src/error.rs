use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("ScoutBuilder error")]
    ScoutBuilder,
    #[error("CargoToml error: {0}")]
    CargoToml(#[from] cargo_toml::Error),
    #[error("Command error: {0}")]
    Command(String),
    #[error("Utf8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("Json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("NotClean error")]
    NotClean,
    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),
}
