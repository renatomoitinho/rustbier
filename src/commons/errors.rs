use crate::commons::Size;
use std::convert::From;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct InvalidSizeError {
    msg: String,
}

impl InvalidSizeError {
    pub fn new(size: &Size) -> InvalidSizeError {
        let message = format!("Size {:?} is not valid.", &size);
        InvalidSizeError { msg: message }
    }
}

impl fmt::Display for InvalidSizeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for InvalidSizeError {
    fn description(&self) -> &str {
        &self.msg
    }
}

impl From<InvalidSizeError> for std::io::Error {
    fn from(error: InvalidSizeError) -> Self {
        std::io::Error::new(std::io::ErrorKind::InvalidData, error)
    }
}

impl From<InvalidSizeError> for opencv::Error {
    fn from(error: InvalidSizeError) -> Self {
        opencv::Error::new(-1, format!("InvalidSizeError: {}", error))
    }
}
