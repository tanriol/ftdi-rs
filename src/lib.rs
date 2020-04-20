//! An incomplete Rust wrapper over the `libftdi1` library for working with FTDI devices
//!
//! Note: the library interface is *definitely* unstable for now

use libftdi1_sys as ffi;

use std::io::{self, Read, Write, ErrorKind};

use num::traits::ToPrimitive;


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


pub struct Context {
    context: *mut ffi::ftdi_context,
}

impl Context {
    pub fn new() -> Context {
        let context = unsafe { ffi::ftdi_new() };
        // Can be null on either OOM or libusb_init failure
        assert!(!context.is_null());

        Context { context }
    }

    /// Do not call after opening the USB device
    pub fn set_interface(&mut self, interface: Interface) -> io::Result<()> {
        let result = unsafe { ffi::ftdi_set_interface(self.context, interface.into()) };
        match result {
            0 => Ok(()),
            -1 => Err(io::Error::new(ErrorKind::InvalidInput, "unknown interface")),
            -2 => Err(io::Error::new(ErrorKind::NotFound, "device not found")),
            -3 => Err(io::Error::new(ErrorKind::Other, "device already opened")),
            _ => Err(io::Error::new(ErrorKind::Other, "unknown set latency error")),
        }
    }

    pub fn usb_open(&mut self, vendor: u16, product: u16) -> io::Result<()> {
        let result = unsafe { ffi::ftdi_usb_open(self.context, vendor as i32, product as i32) };
        match result {
            0 => Ok(()),
            -3 => Err(io::Error::new(ErrorKind::NotFound, "device not found")),
            -4 => Err(io::Error::new(ErrorKind::Other, "unable to open device")),
            -5 => Err(io::Error::new(ErrorKind::Other, "unable to claim device")),
            -6 => Err(io::Error::new(ErrorKind::Other, "reset failed")),
            -7 => Err(io::Error::new(ErrorKind::Other, "set baudrate failed")),
            -8 => Err(io::Error::new(ErrorKind::Other, "get description failed")),
            -9 => Err(io::Error::new(ErrorKind::Other, "get serial failed")),
            -12 => Err(io::Error::new(ErrorKind::Other, "libusb_get_device_list failed")),
            -13 => Err(io::Error::new(ErrorKind::Other, "libusb_get_device_descriptor failed")),
            _ => Err(io::Error::new(ErrorKind::Other, "unknown usb_open error")),
        }
    }

    pub fn usb_reset(&mut self) -> io::Result<()> {
        let result = unsafe { ffi::ftdi_usb_reset(self.context) };
        match result {
            0 => Ok(()),
            -1 => Err(io::Error::new(ErrorKind::Other, "reset failed")),
            -2 => Err(io::Error::new(ErrorKind::NotFound, "device not found")),
            _ => Err(io::Error::new(ErrorKind::Other, "unknown reset error")),
        }
    }

    pub fn usb_purge_buffers(&mut self) -> io::Result<()> {
        let result = unsafe { ffi::ftdi_usb_purge_buffers(self.context) };
        match result {
            0 => Ok(()),
            -1 => Err(io::Error::new(ErrorKind::Other, "read purge failed")),
            -2 => Err(io::Error::new(ErrorKind::Other, "write purge failed")),
            -3 => Err(io::Error::new(ErrorKind::NotFound, "device not found")),
            _ => Err(io::Error::new(ErrorKind::Other, "unknown purge error")),
        }
    }

    pub fn set_latency_timer(&mut self, value: u8) -> io::Result<()> {
        let result = unsafe { ffi::ftdi_set_latency_timer(self.context, value) };
        match result {
            0 => Ok(()),
            -1 => Err(io::Error::new(ErrorKind::InvalidInput, "bad latency value")),
            -2 => Err(io::Error::new(ErrorKind::Other, "set latency failed")),
            -3 => Err(io::Error::new(ErrorKind::NotFound, "device not found")),
            _ => Err(io::Error::new(ErrorKind::Other, "unknown set latency error")),
        }
    }

    pub fn latency_timer(&mut self) -> io::Result<u8> {
        let mut value = 0u8;
        let result = unsafe { ffi::ftdi_get_latency_timer(self.context, &mut value) };
        match result {
            0 => Ok(value),
            -1 => Err(io::Error::new(ErrorKind::Other, "set latency failed")),
            -2 => Err(io::Error::new(ErrorKind::NotFound, "device not found")),
            _ => Err(io::Error::new(ErrorKind::Other, "unknown get latency error")),
        }
    }

    pub fn set_write_chunksize(&mut self, value: u32) {
        let result = unsafe {
            ffi::ftdi_write_data_set_chunksize(self.context, value)
        };
        match result {
            0 => (),
            err => panic!("unknown set_write_chunksize retval {:?}", err)
        }
    }

    pub fn write_chunksize(&mut self) -> u32 {
        let mut value = 0;
        let result = unsafe {
            ffi::ftdi_write_data_get_chunksize(self.context, &mut value)
        };
        match result {
            0 => value,
            err => panic!("unknown get_write_chunksize retval {:?}", err)
        }
    }

    pub fn set_read_chunksize(&mut self, value: u32) {
        let result = unsafe {
            ffi::ftdi_read_data_set_chunksize(self.context, value)
        };
        match result {
            0 => (),
            err => panic!("unknown set_write_chunksize retval {:?}", err)
        }
    }

    pub fn read_chunksize(&mut self) -> u32 {
        let mut value = 0;
        let result = unsafe {
            ffi::ftdi_read_data_get_chunksize(self.context, &mut value)
        };
        match result {
            0 => value,
            err => panic!("unknown get_write_chunksize retval {:?}", err)
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { ffi::ftdi_free(self.context) }
    }
}

impl Read for Context {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = buf.len().to_i32().unwrap_or(std::i32::MAX);
        let result = unsafe { ffi::ftdi_read_data(self.context, buf.as_mut_ptr(), len) };
        match result {
            count if count >= 0 => Ok(count as usize),
            -666 => Err(io::Error::new(ErrorKind::NotFound, "device not found in read")),
            libusb_error => {
                Err(io::Error::new(ErrorKind::Other,
                                   format!("libusb_bulk_transfer error {}", libusb_error)))
            }
        }
    }
}

impl Write for Context {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = buf.len().to_i32().unwrap_or(std::i32::MAX);
        let result = unsafe { ffi::ftdi_write_data(self.context, buf.as_ptr(), len) };
        match result {
            count if count >= 0 => Ok(count as usize),
            -666 => Err(io::Error::new(ErrorKind::NotFound, "device not found in write")),
            libusb_error => {
                Err(io::Error::new(ErrorKind::Other,
                                   format!("usb_bulk_write error {}", libusb_error)))
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

