//! An incomplete Rust wrapper over the `libftdi1` library for working with FTDI devices
//!
//! Note: the library interface is *definitely* unstable for now

extern crate num;

extern crate libftdi1_sys as ffi;

use std::io;
use std::io::{ErrorKind, Read, Write};

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

pub enum BitMode {
    RESET,
    BITBANG,
    MPSSE,
    SYNCBB,
    MCU,
    OPTO,
    CBUS,
    SYNCFF,
    FT1284,
}

impl Into<u8> for BitMode {
    fn into(self) -> u8 {
        let mode = match self {
            BitMode::RESET => ffi::ftdi_mpsse_mode::BITMODE_RESET,
            BitMode::BITBANG => ffi::ftdi_mpsse_mode::BITMODE_BITBANG,
            BitMode::MPSSE => ffi::ftdi_mpsse_mode::BITMODE_MPSSE,
            BitMode::SYNCBB => ffi::ftdi_mpsse_mode::BITMODE_SYNCBB,
            BitMode::MCU => ffi::ftdi_mpsse_mode::BITMODE_MCU,
            BitMode::OPTO => ffi::ftdi_mpsse_mode::BITMODE_OPTO,
            BitMode::CBUS => ffi::ftdi_mpsse_mode::BITMODE_CBUS,
            BitMode::SYNCFF => ffi::ftdi_mpsse_mode::BITMODE_SYNCFF,
            BitMode::FT1284 => ffi::ftdi_mpsse_mode::BITMODE_FT1284,
        };

        mode as u8
    }
}

#[allow(non_camel_case_types)]
pub enum FlowControl {
    SIO_DISABLE_FLOW_CTRL,
    SIO_RTS_CTS_HS,
    SIO_DTR_DSR_HS,
    SIO_XON_XOFF_HS,
}

impl Into<i32> for FlowControl {
    fn into(self) -> i32 {
        match self {
            FlowControl::SIO_DISABLE_FLOW_CTRL => 0x0,
            FlowControl::SIO_RTS_CTS_HS => (0x1 << 8),
            FlowControl::SIO_DTR_DSR_HS => (0x2 << 8),
            FlowControl::SIO_XON_XOFF_HS => (0x4 << 8),
        }
    }
}

pub struct Context {
    native: ffi::ftdi_context,
}

impl Context {
    pub fn new() -> Context {
        let mut context = Context {
            native: Default::default(),
        };
        let result = unsafe { ffi::ftdi_init(&mut context.native) };
        // Can be non-zero on either OOM or libusb_init failure
        assert!(result == 0);
        context
    }

    /// Do not call after opening the USB device
    pub fn set_interface(&mut self, interface: Interface) -> io::Result<()> {
        let result = unsafe { ffi::ftdi_set_interface(&mut self.native, interface.into()) };
        match result {
            0 => Ok(()),
            -1 => Err(io::Error::new(ErrorKind::InvalidInput, "unknown interface")),
            -2 => Err(io::Error::new(ErrorKind::NotFound, "device not found")),
            -3 => Err(io::Error::new(ErrorKind::Other, "device already opened")),
            _ => Err(io::Error::new(
                ErrorKind::Other,
                "unknown set latency error",
            )),
        }
    }

    pub fn usb_open(&mut self, vendor: u16, product: u16) -> io::Result<()> {
        let result = unsafe { ffi::ftdi_usb_open(&mut self.native, vendor as i32, product as i32) };
        match result {
            0 => Ok(()),
            -3 => Err(io::Error::new(ErrorKind::NotFound, "device not found")),
            -4 => Err(io::Error::new(ErrorKind::Other, "unable to open device")),
            -5 => Err(io::Error::new(ErrorKind::Other, "unable to claim device")),
            -6 => Err(io::Error::new(ErrorKind::Other, "reset failed")),
            -7 => Err(io::Error::new(ErrorKind::Other, "set baudrate failed")),
            -8 => Err(io::Error::new(ErrorKind::Other, "get description failed")),
            -9 => Err(io::Error::new(ErrorKind::Other, "get serial failed")),
            -12 => Err(io::Error::new(
                ErrorKind::Other,
                "libusb_get_device_list failed",
            )),
            -13 => Err(io::Error::new(
                ErrorKind::Other,
                "libusb_get_device_descriptor failed",
            )),
            _ => Err(io::Error::new(ErrorKind::Other, "unknown usb_open error")),
        }
    }

    pub fn usb_reset(&mut self) -> io::Result<()> {
        let result = unsafe { ffi::ftdi_usb_reset(&mut self.native) };
        match result {
            0 => Ok(()),
            -1 => Err(io::Error::new(ErrorKind::Other, "reset failed")),
            -2 => Err(io::Error::new(ErrorKind::NotFound, "device not found")),
            _ => Err(io::Error::new(ErrorKind::Other, "unknown reset error")),
        }
    }

    pub fn usb_purge_buffers(&mut self) -> io::Result<()> {
        let result = unsafe { ffi::ftdi_usb_purge_buffers(&mut self.native) };
        match result {
            0 => Ok(()),
            -1 => Err(io::Error::new(ErrorKind::Other, "read purge failed")),
            -2 => Err(io::Error::new(ErrorKind::Other, "write purge failed")),
            -3 => Err(io::Error::new(ErrorKind::NotFound, "device not found")),
            _ => Err(io::Error::new(ErrorKind::Other, "unknown purge error")),
        }
    }

    pub fn set_latency_timer(&mut self, value: u8) -> io::Result<()> {
        let result = unsafe { ffi::ftdi_set_latency_timer(&mut self.native, value) };
        match result {
            0 => Ok(()),
            -1 => Err(io::Error::new(ErrorKind::InvalidInput, "bad latency value")),
            -2 => Err(io::Error::new(ErrorKind::Other, "set latency failed")),
            -3 => Err(io::Error::new(ErrorKind::NotFound, "device not found")),
            _ => Err(io::Error::new(
                ErrorKind::Other,
                "unknown set latency error",
            )),
        }
    }

    pub fn latency_timer(&mut self) -> io::Result<u8> {
        let mut value = 0u8;
        let result = unsafe { ffi::ftdi_get_latency_timer(&mut self.native, &mut value) };
        match result {
            0 => Ok(value),
            -1 => Err(io::Error::new(ErrorKind::Other, "set latency failed")),
            -2 => Err(io::Error::new(ErrorKind::NotFound, "device not found")),
            _ => Err(io::Error::new(
                ErrorKind::Other,
                "unknown get latency error",
            )),
        }
    }

    pub fn set_write_chunksize(&mut self, value: u32) {
        let result = unsafe { ffi::ftdi_write_data_set_chunksize(&mut self.native, value) };
        match result {
            0 => (),
            err => panic!("unknown set_write_chunksize retval {:?}", err),
        }
    }

    pub fn write_chunksize(&mut self) -> u32 {
        let mut value = 0;
        let result = unsafe { ffi::ftdi_write_data_get_chunksize(&mut self.native, &mut value) };
        match result {
            0 => value,
            err => panic!("unknown get_write_chunksize retval {:?}", err),
        }
    }

    pub fn set_read_chunksize(&mut self, value: u32) {
        let result = unsafe { ffi::ftdi_read_data_set_chunksize(&mut self.native, value) };
        match result {
            0 => (),
            err => panic!("unknown set_write_chunksize retval {:?}", err),
        }
    }

    pub fn read_chunksize(&mut self) -> u32 {
        let mut value = 0;
        let result = unsafe { ffi::ftdi_read_data_get_chunksize(&mut self.native, &mut value) };
        match result {
            0 => value,
            err => panic!("unknown get_write_chunksize retval {:?}", err),
        }
    }

    pub fn set_flow_control(&mut self, flowctrl: FlowControl) -> io::Result<()> {
        let result = unsafe { ffi::ftdi_setflowctrl(&mut self.native, flowctrl.into()) };
        match result {
            0 => Ok(()),
            -1 => Err(io::Error::new(ErrorKind::Other, "set flow control failed")),
            -2 => Err(io::Error::new(ErrorKind::NotFound, "device not found")),
            _ => Err(io::Error::new(
                ErrorKind::Other,
                "unknown set flow control error",
            )),
        }
    }

    pub fn set_bitmode(&mut self, bitmask: u8, mode: BitMode) -> io::Result<()> {
        let result = unsafe { ffi::ftdi_set_bitmode(&mut self.native, bitmask, mode.into()) };
        match result {
            0 => Ok(()),
            -1 => Err(io::Error::new(ErrorKind::Other, "set bitmode failed")),
            -2 => Err(io::Error::new(ErrorKind::NotFound, "device not found")),
            _ => Err(io::Error::new(
                ErrorKind::Other,
                "unknown set bitmode error",
            )),
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { ffi::ftdi_deinit(&mut self.native) }
    }
}

impl Read for Context {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = buf.len().to_i32().unwrap_or(std::i32::MAX);
        let result = unsafe { ffi::ftdi_read_data(&mut self.native, buf.as_mut_ptr(), len) };
        match result {
            count if count >= 0 => Ok(count as usize),
            -666 => Err(io::Error::new(
                ErrorKind::NotFound,
                "device not found in read",
            )),
            libusb_error => Err(io::Error::new(
                ErrorKind::Other,
                format!("libusb_bulk_transfer error {}", libusb_error),
            )),
        }
    }
}

impl Write for Context {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = buf.len().to_i32().unwrap_or(std::i32::MAX);
        let result = unsafe { ffi::ftdi_write_data(&mut self.native, buf.as_ptr(), len) };
        match result {
            count if count >= 0 => Ok(count as usize),
            -666 => Err(io::Error::new(
                ErrorKind::NotFound,
                "device not found in write",
            )),
            libusb_error => Err(io::Error::new(
                ErrorKind::Other,
                format!("usb_bulk_write error {}", libusb_error),
            )),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn it_works() {}
}
