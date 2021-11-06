use thiserror::Error;

use std::ffi::CStr;
use std::io;

use super::ffi;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to enumerate devices to open the correct one")]
    EnumerationFailed,
    #[error("the specified device could not be found")]
    DeviceNotFound,
    #[error("libftdi context allocation failed")]
    AllocationFailed,
    #[error("failed to open the specified device")]
    AccessFailed,
    #[error("the requested interface could not be claimed")]
    ClaimFailed,
    #[error("the device has been disconnected from the system")]
    Disconnected,
    #[error("the device does not have the specified interface")]
    NoSuchInterface,
    #[error("libftdi reported error to perform operation")]
    RequestFailed,
    #[error("input value invalid: {0}")]
    InvalidInput(&'static str),

    #[error("unknown or unexpected libftdi error")]
    Unknown { source: LibFtdiError },

    #[error("INTERNAL, DO NOT USE")]
    #[doc(hidden)]
    __NonExhaustive,
}

impl Error {
    pub(crate) fn unknown(context: *mut ffi::ftdi_context) -> Self {
        let message = unsafe { CStr::from_ptr(ffi::ftdi_get_error_string(context)) }
            .to_str()
            .expect("all error strings are expected to be ASCII");
        Error::Unknown {
            source: LibFtdiError { message },
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
#[error("libftdi: {message}")]
pub struct LibFtdiError {
    message: &'static str,
}

// Ideally this should be using libusb bindings, but we don't depend on any specific USB crate yet
pub(crate) fn libusb_to_io(code: i32) -> io::Error {
    io::Error::new(io::ErrorKind::Other, format!("libusb error code {}", code))
}

pub(crate) fn libftdi_to_io(err: Error) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err)
}
