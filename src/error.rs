#[derive(Debug)]
pub enum Error {
    Command(String),
    Utf8(std::string::FromUtf8Error),
    Json(serde_json::Error),
    NotClean,
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
