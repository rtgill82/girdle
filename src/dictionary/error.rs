// 
// Copyright (c) 2022, Robert Gill <rtgill82@gmail.com>
// 

use std::convert;
use std::error;
use std::fmt;
use std::io;
use std::result;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub struct Error
{
    msg: String,
    source: Option<io::Error>,
}

impl Error {
    pub fn new(msg: &str) -> Error {
        Error {
            msg: String::from(msg),
            source: None
        }
    }
}

impl convert::From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error {
            msg: String::from("IO Error"),
            source: Some(error),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        self.source.as_ref().map(|v| v as &dyn error::Error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.source {
            None => write!(f, "{}", self.msg),
            Some(error) => write!(f, "{}: {}", self.msg, error)
        }
    }
}
