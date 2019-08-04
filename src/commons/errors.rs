use crate::commons::Size;
use std::convert::From;
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub struct InvalidSizeError {
    msg: String,
}

#[derive(Debug, PartialEq)]
pub struct MagickError {
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

impl fmt::Display for MagickError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for InvalidSizeError {
    fn description(&self) -> &str {
        &self.msg
    }
}

impl Error for MagickError {
    fn description(&self) -> &str {
        &self.msg
    }
}

impl From<InvalidSizeError> for std::io::Error {
    fn from(error: InvalidSizeError) -> Self {
        std::io::Error::new(std::io::ErrorKind::InvalidData, error)
    }
}

impl From<InvalidSizeError> for String {
    fn from(error: InvalidSizeError) -> Self {
        format!("InvalidSizeError: {}", error)
    }
}

impl From<InvalidSizeError> for MagickError {
    fn from(error: InvalidSizeError) -> Self {
        MagickError { msg: error.msg }
    }
}

impl From<MagickError> for String {
    fn from(error: MagickError) -> Self {
        format!("MagickError: {}", error)
    }
}

impl From<&str> for MagickError {
    fn from(error: &str) -> Self {
        MagickError {
            msg: String::from(error),
        }
    }
}

impl From<InvalidSizeError> for opencv::Error {
    fn from(error: InvalidSizeError) -> Self {
        opencv::Error::new(-1, format!("InvalidSizeError: {}", error))
    }
}

impl From<MagickError> for opencv::Error {
    fn from(error: MagickError) -> Self {
        opencv::Error::new(-1, format!("MagickError: {}", error))
    }
}

impl From<MagickError> for actix_web::Error {
    fn from(error: MagickError) -> Self {
        actix_web::error::ErrorInternalServerError(error)
    }
}
