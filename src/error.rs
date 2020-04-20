use thiserror::Error;

use std::ffi::CStr;

use super::ffi;

#[derive(Debug, Error)]
pub enum Error {
    #[error("the specified device could not be found")]
    DeviceNotFound,
    #[error("the device has been disconnected from the system")]
    Disconnected,
    #[error("the device does not have the specified interface")]
    NoSuchInterface,
    #[error("libftdi reported error to perform operation")]
    RequestFailed,
    #[error("input value invalid: {0}")]
    InvalidInput(&'static str),

    #[error("unknown or unexpected libftdi error: {message}")]
    Unknown { message: &'static str },

    #[error("INTERNAL, DO NOT USE")]
    #[doc(hidden)]
    __NonExhaustive,
}

impl Error {
    pub(crate) fn unknown(context: *mut ffi::ftdi_context) -> Self {
        let message = unsafe { CStr::from_ptr(ffi::ftdi_get_error_string(context)) }
            .to_str()
            .expect("all error strings are expected to be ASCII");
        Self::Unknown { message }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
