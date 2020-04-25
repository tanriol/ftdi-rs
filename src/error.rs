use thiserror::Error;

use std::ffi::CStr;
use std::io;
use std::str;

use super::ffi;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to initialize the ftdi context")]
    InitFailed,
    #[error("failed to enumerate devices to open the correct one")]
    EnumerationFailed,
    #[error("the specified device could not be found")]
    DeviceNotFound,
    #[error("failed to open or close the specified device")]
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
        let message = unsafe {
            // Null pointer returns empty string. And we otherwise can't
            // use a context without Builder::new() returning first.
            let err_raw = ffi::ftdi_get_error_string(context);
            // Manually checked- every error string in libftdi1 is ASCII.
            str::from_utf8_unchecked(CStr::from_ptr(err_raw).to_bytes())
        };

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

#[derive(Debug, Error)]
#[error("libusb error code {code}")]
pub struct LibUsbError {
    code: i32,
}

// Ideally this should be using libusb bindings, but we don't depend on any specific USB crate yet
pub(crate) fn libusb_to_io(code: i32) -> io::Error {
    io::Error::new(io::ErrorKind::Other, LibUsbError { code })
}
