//! An incomplete Rust wrapper over the `libftdi1` library for working with FTDI devices
//!
//! Note: the library interface is *definitely* unstable for now

use libftdi1_sys as ffi;

use std::convert::TryInto;
use std::io::{self, Read, Write};

pub mod error;

pub use error::{Error, Result};

/// The target interface
pub enum Interface {
    A,
    B,
    C,
    D,
    Any,
}

impl Into<ffi::ftdi_interface> for Interface {
    fn into(self) -> ffi::ftdi_interface {
        match self {
            Interface::A => ffi::ftdi_interface::INTERFACE_A,
            Interface::B => ffi::ftdi_interface::INTERFACE_B,
            Interface::C => ffi::ftdi_interface::INTERFACE_C,
            Interface::D => ffi::ftdi_interface::INTERFACE_D,
            Interface::Any => ffi::ftdi_interface::INTERFACE_ANY,
        }
    }
}

pub struct Builder {
    context: *mut ffi::ftdi_context,
}

impl Builder {
    pub fn new() -> Result<Self> {
        let context = unsafe { ffi::ftdi_new() };

        // Can be non-zero on either OOM or libusb_init failure
        if context.is_null() {
            Err(Error::InitFailed)
        } else {
            Ok(Self { context })
        }
    }

    pub fn set_interface(&mut self, interface: Interface) -> Result<()> {
        let result = unsafe { ffi::ftdi_set_interface(self.context, interface.into()) };
        match result {
            0 => Ok(()),
            -1 => unreachable!("unknown interface from ftdi.h"),
            -2 => unreachable!("missing context"),
            -3 => unreachable!("device already opened in Builder"),
            _ => Err(Error::unknown(self.context)),
        }
    }

    pub fn usb_open(self, vendor: u16, product: u16) -> Result<Device> {
        let result = unsafe { ffi::ftdi_usb_open(self.context, vendor as i32, product as i32) };
        match result {
            0 => Ok(Device {
                context: self.context,
            }),
            -1 => Err(Error::EnumerationFailed), // usb_find_busses() failed
            -2 => Err(Error::EnumerationFailed), // usb_find_devices() failed
            -3 => Err(Error::DeviceNotFound),    // usb device not found
            -4 => Err(Error::AccessFailed),      // unable to open device
            -5 => Err(Error::ClaimFailed),       // unable to claim device
            -6 => Err(Error::RequestFailed),     // reset failed
            -7 => Err(Error::RequestFailed),     // set baudrate failed
            -8 => Err(Error::EnumerationFailed), // get product description failed
            -9 => Err(Error::EnumerationFailed), // get serial number failed
            -10 => Err(Error::unknown(self.context)), // unable to close device
            -11 => unreachable!("uninitialized context"), // ftdi context invalid
            -12 => Err(Error::EnumerationFailed), // libusb_get_device_list() failed
            _ => Err(Error::unknown(self.context)),
        }
    }
}

impl Drop for Builder {
    fn drop(&mut self) {
        unsafe { ffi::ftdi_free(self.context) }
    }
}

pub struct Device {
    context: *mut ffi::ftdi_context,
}

impl Device {
    pub fn usb_reset(&mut self) -> Result<()> {
        let result = unsafe { ffi::ftdi_usb_reset(self.context) };
        match result {
            0 => Ok(()),
            -1 => Err(Error::RequestFailed),
            -2 => unreachable!("uninitialized context"),
            _ => Err(Error::unknown(self.context)),
        }
    }

    pub fn usb_purge_buffers(&mut self) -> Result<()> {
        let result = unsafe { ffi::ftdi_usb_purge_buffers(self.context) };
        match result {
            0 => Ok(()),
            -1 /* read */ | -2 /* write */ => Err(Error::RequestFailed),
            -3 => unreachable!("uninitialized context"),
            _ => Err(Error::unknown(self.context)),
        }
    }

    pub fn set_latency_timer(&mut self, value: u8) -> Result<()> {
        let result = unsafe { ffi::ftdi_set_latency_timer(self.context, value) };
        match result {
            0 => Ok(()),
            -1 => Err(Error::InvalidInput("latency value out of range")),
            -2 => Err(Error::RequestFailed),
            -3 => unreachable!("uninitialized context"),
            _ => Err(Error::unknown(self.context)),
        }
    }

    pub fn latency_timer(&mut self) -> Result<u8> {
        let mut value = 0u8;
        let result = unsafe { ffi::ftdi_get_latency_timer(self.context, &mut value) };
        match result {
            0 => Ok(value),
            -1 => Err(Error::RequestFailed),
            -2 => unreachable!("uninitialized context"),
            _ => Err(Error::unknown(self.context)),
        }
    }

    pub fn set_write_chunksize(&mut self, value: u32) {
        let result = unsafe { ffi::ftdi_write_data_set_chunksize(self.context, value) };
        match result {
            0 => (),
            -1 => unreachable!("uninitialized context"),
            err => panic!("unknown set_write_chunksize retval {:?}", err),
        }
    }

    pub fn write_chunksize(&mut self) -> u32 {
        let mut value = 0;
        let result = unsafe { ffi::ftdi_write_data_get_chunksize(self.context, &mut value) };
        match result {
            0 => value,
            -1 => unreachable!("uninitialized context"),
            err => panic!("unknown get_write_chunksize retval {:?}", err),
        }
    }

    pub fn set_read_chunksize(&mut self, value: u32) {
        let result = unsafe { ffi::ftdi_read_data_set_chunksize(self.context, value) };
        match result {
            0 => (),
            -1 => unreachable!("uninitialized context"),
            err => panic!("unknown set_write_chunksize retval {:?}", err),
        }
    }

    pub fn read_chunksize(&mut self) -> u32 {
        let mut value = 0;
        let result = unsafe { ffi::ftdi_read_data_get_chunksize(self.context, &mut value) };
        match result {
            0 => value,
            -1 => unreachable!("uninitialized context"),
            err => panic!("unknown get_write_chunksize retval {:?}", err),
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe { ffi::ftdi_free(self.context) }
    }
}

impl Read for Device {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = buf.len().try_into().unwrap_or(std::i32::MAX);
        let result = unsafe { ffi::ftdi_read_data(self.context, buf.as_mut_ptr(), len) };
        match result {
            count if count >= 0 => Ok(count as usize),
            -666 => unreachable!("uninitialized context"),
            err => Err(error::libusb_to_io(err)),
        }
    }
}

impl Write for Device {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = buf.len().try_into().unwrap_or(std::i32::MAX);
        let result = unsafe { ffi::ftdi_write_data(self.context, buf.as_ptr(), len) };
        match result {
            count if count >= 0 => Ok(count as usize),
            -666 => unreachable!("uninitialized context"),
            err => Err(error::libusb_to_io(err)),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
