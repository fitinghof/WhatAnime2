use serde::ser::StdError;
#[derive(Debug)]

pub type Result<T> = std::result::Result<T, Error>;

pub enum Error {
    ParseError(String),
}
