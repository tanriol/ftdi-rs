use std::convert::TryFrom;
use std::error;
use std::ffi::CStr;
use std::fmt;
use std::os::raw;
use std::str::Utf8Error;

#[derive(Debug)]
pub enum Error {
    LibFtdi(LibFtdiError),
    LibUsb(LibUsbError),
    MallocFailure,
    UnexpectedErrorCode(raw::c_int),
    Utf8(Utf8Error),
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
            Error::Utf8(_) => write!(f, "libftdi error string not UTF8"),
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

pub(super) struct LibFtdiReturn<'a>(raw::c_int, &'a super::Context);

impl<'a> LibFtdiReturn<'a> {
    pub(super) fn new(rc: raw::c_int, ctx: &'a super::Context) -> Self {
        LibFtdiReturn(rc, ctx)
    }
}

impl<'a> Into<Result<u32, Error>> for LibFtdiReturn<'a> {
    fn into(self) -> Result<u32, Error> {
        match u32::try_from(self.0) {
            // In libftdi1, return codes >= 0 are success.
            Ok(v) => Ok(v),
            Err(_) => match self.0 {
                -13..=-1 | -666 => {
                    let err_raw = unsafe { super::ffi::ftdi_get_error_string(self.1.native) };

                    match unsafe { CStr::from_ptr(err_raw) }.to_str() {
                        Ok(err_str) => Err(Error::LibFtdi(LibFtdiError { err_str })),
                        Err(utf8_err) => Err(Error::Utf8(utf8_err)),
                    }
                }
                unk => Err(Error::UnexpectedErrorCode(unk)),
            },
        }
    }
}

impl<'a> Into<Result<(), Error>> for LibFtdiReturn<'a> {
    fn into(self) -> Result<(), Error> {
        match u32::try_from(self.0) {
            // In libftdi1, return codes >= 0 are success.
            Ok(_) => Ok(()),
            Err(_) => match self.0 {
                -13..=-1 | -666 => {
                    let err_raw = unsafe { super::ffi::ftdi_get_error_string(self.1.native) };

                    match unsafe { CStr::from_ptr(err_raw) }.to_str() {
                        Ok(err_str) => Err(Error::LibFtdi(LibFtdiError { err_str })),
                        Err(utf8_err) => Err(Error::Utf8(utf8_err)),
                    }
                }
                unk => Err(Error::UnexpectedErrorCode(unk)),
            },
        }
    }
}

impl std::error::Error for LibFtdiError {}
impl std::error::Error for LibUsbError {}
