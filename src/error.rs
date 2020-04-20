use std::error;
use std::fmt;
use std::os::raw;
use std::result;
use std::str;

#[derive(Debug)]
pub enum Error {
    LibFtdi(LibFtdiError),
    LibUsb(LibUsbError),
    MallocFailure,
    UnexpectedErrorCode(raw::c_int),
}

#[derive(Debug)]
pub struct LibFtdiError {
    err_str: &'static str,
}

impl LibFtdiError {
    pub fn new(err_str: &'static str) -> LibFtdiError {
        LibFtdiError { err_str }
    }
}

#[derive(Debug)]
pub struct LibUsbError {
    err_str: &'static str,
}

// LibUsbError is a wrapper for the libusb-specific error types
// returned by libftdi1, in case someone ever decides to implement
// ftdi-rs directly over libusb.
impl LibUsbError {
    pub fn new(err_str: &'static str) -> LibUsbError {
        LibUsbError { err_str }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::LibFtdi(_) => write!(f, "libftdi-internal error"),
            Error::LibUsb(_) => write!(f, "libusb-internal error"),
            Error::MallocFailure => write!(f, "malloc() failure"),
            Error::UnexpectedErrorCode(c) => write!(f, "unknown libftdi error code {}", c),
        }
    }
}

impl fmt::Display for LibFtdiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.err_str)
    }
}

impl fmt::Display for LibUsbError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.err_str)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Error::LibFtdi(ref ftdi_err) => Some(ftdi_err),
            _ => None,
        }
    }
}

pub type Result<T> = result::Result<T, Error>;

impl std::error::Error for LibFtdiError {}
impl std::error::Error for LibUsbError {}
