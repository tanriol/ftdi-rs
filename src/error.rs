use std::convert::TryFrom;
use std::error;
use std::ffi::CStr;
use std::fmt;
use std::os::raw;
use std::str;
use std::result;

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

pub(super) struct LibFtdiReturn<'a>(raw::c_int, &'a super::Context);

impl<'a> LibFtdiReturn<'a> {
    pub(super) fn new(rc: raw::c_int, ctx: &'a super::Context) -> Self {
        LibFtdiReturn(rc, ctx)
    }
}

impl<'a> Into<Result<u32>> for LibFtdiReturn<'a> {
    fn into(self) -> Result<u32> {
        match u32::try_from(self.0) {
            // In libftdi1, return codes >= 0 are success.
            Ok(v) => Ok(v),
            Err(_) => match self.0 {
                -13..=-1 | -666 => {
                    let err_str = unsafe {
                        let err_raw = super::ffi::ftdi_get_error_string(self.1.native);
                        // Manually checked- every error string in libftdi1 is ASCII.
                        str::from_utf8_unchecked(CStr::from_ptr(err_raw).to_bytes())
                    };

                    Err(Error::LibFtdi(LibFtdiError { err_str }))
                }
                unk => Err(Error::UnexpectedErrorCode(unk)),
            },
        }
    }
}

impl<'a> Into<Result<()>> for LibFtdiReturn<'a> {
    fn into(self) -> Result<()> {
        match u32::try_from(self.0) {
            // In libftdi1, return codes >= 0 are success.
            Ok(_) => Ok(()),
            Err(_) => match self.0 {
                -13..=-1 | -666 => {
                    let err_str = unsafe {
                        let err_raw = super::ffi::ftdi_get_error_string(self.1.native);
                        // Manually checked- every error string in libftdi1 is ASCII.
                        str::from_utf8_unchecked(CStr::from_ptr(err_raw).to_bytes())
                    };

                    Err(Error::LibFtdi(LibFtdiError { err_str }))
                }
                unk => Err(Error::UnexpectedErrorCode(unk)),
            },
        }
    }
}

impl std::error::Error for LibFtdiError {}
impl std::error::Error for LibUsbError {}
