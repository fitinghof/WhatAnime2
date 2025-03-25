pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ParseError(String),
    ReqwestError(reqwest::Error),
    UnsuccessfulResponse {
        status: reqwest::StatusCode,
        text: String,
    },
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::ReqwestError(value)
    }
}
