use std::fmt;
use std::error;

#[derive(Debug)]
pub enum Error {
    LibFtdi(LibFtdiError),
    MallocFailure,
}

#[derive(Debug)]
pub struct LibFtdiError {
    err_str : &'static str,
}

impl LibFtdiError {
    pub fn new(err_str : &'static str) -> LibFtdiError {
        LibFtdiError {
            err_str : err_str,
        }
    }
}


impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::LibFtdi(_) => {
                write!(f, "libftdi-internal error")
            },
            Error::MallocFailure => {
                write!(f, "malloc() failure")
            }
        }
    }
}

impl fmt::Display for LibFtdiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.err_str)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Error::LibFtdi(ref ftdi_err) => {
                Some(ftdi_err)
            },
            Error::MallocFailure => {
                None
            }
        }
    }
}

impl std::error::Error for LibFtdiError {}
