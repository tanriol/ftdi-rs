//! An incomplete Rust wrapper over the `libftdi1` library for working with FTDI devices
//!
//! Note: the library interface is *definitely* unstable for now

extern crate num;

extern crate libftdi1_sys as ffi;

use std::io;
use std::io::{Read, Write, ErrorKind};

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


pub enum MpsseMode {
    Reset,
    Bitbang,
    Mpsse,
    SyncBb,
    Mcu,
    Opto,
    Cbus,
    SyncFf,
    Ft1284,
}

impl Into<ffi::ftdi_mpsse_mode> for MpsseMode {
    fn into(self) -> ffi::ftdi_mpsse_mode {
        match self {
            MpsseMode::Reset => ffi::ftdi_mpsse_mode::BITMODE_RESET,
            MpsseMode::Bitbang => ffi::ftdi_mpsse_mode::BITMODE_BITBANG,
            MpsseMode::Mpsse => ffi::ftdi_mpsse_mode::BITMODE_MPSSE,
            MpsseMode::SyncBb => ffi::ftdi_mpsse_mode::BITMODE_SYNCBB,
            MpsseMode::Mcu => ffi::ftdi_mpsse_mode::BITMODE_MCU,
            MpsseMode::Opto => ffi::ftdi_mpsse_mode::BITMODE_OPTO,
            MpsseMode::Cbus => ffi::ftdi_mpsse_mode::BITMODE_CBUS,
            MpsseMode::SyncFf => ffi::ftdi_mpsse_mode::BITMODE_SYNCFF,
            MpsseMode::Ft1284 => ffi::ftdi_mpsse_mode::BITMODE_FT1284,
        }
    }
}


pub struct Context {
    native: ffi::ftdi_context,
}

impl Context {
    pub fn new() -> Context {
        let mut context = Context { native: Default::default() };
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
            _ => Err(io::Error::new(ErrorKind::Other, "unknown set latency error")),
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
            -12 => Err(io::Error::new(ErrorKind::Other, "libusb_get_device_list failed")),
            -13 => Err(io::Error::new(ErrorKind::Other, "libusb_get_device_descriptor failed")),
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
            _ => Err(io::Error::new(ErrorKind::Other, "unknown set latency error")),
        }
    }

    pub fn latency_timer(&mut self) -> io::Result<u8> {
        let mut value = 0u8;
        let result = unsafe { ffi::ftdi_get_latency_timer(&mut self.native, &mut value) };
        match result {
            0 => Ok(value),
            -1 => Err(io::Error::new(ErrorKind::Other, "set latency failed")),
            -2 => Err(io::Error::new(ErrorKind::NotFound, "device not found")),
            _ => Err(io::Error::new(ErrorKind::Other, "unknown get latency error")),
        }
    }

    pub fn set_write_chunksize(&mut self, value: u32) {
        let result = unsafe {
            ffi::ftdi_write_data_set_chunksize(&mut self.native, value)
        };
        match result {
            0 => (),
            err => panic!("unknown set_write_chunksize retval {:?}", err)
        }
    }

    pub fn write_chunksize(&mut self) -> u32 {
        let mut value = 0;
        let result = unsafe {
            ffi::ftdi_write_data_get_chunksize(&mut self.native, &mut value)
        };
        match result {
            0 => value,
            err => panic!("unknown get_write_chunksize retval {:?}", err)
        }
    }

    pub fn set_read_chunksize(&mut self, value: u32) {
        let result = unsafe {
            ffi::ftdi_read_data_set_chunksize(&mut self.native, value)
        };
        match result {
            0 => (),
            err => panic!("unknown set_write_chunksize retval {:?}", err)
        }
    }

    pub fn read_chunksize(&mut self) -> u32 {
        let mut value = 0;
        let result = unsafe {
            ffi::ftdi_read_data_get_chunksize(&mut self.native, &mut value)
        };
        match result {
            0 => value,
            err => panic!("unknown get_write_chunksize retval {:?}", err)
        }
    }

    pub fn set_baudrate(&mut self, baudrate: i32) -> io::Result<()> {
        let result = unsafe { ffi::ftdi_set_baudrate(&mut self.native, baudrate) };
        match result {
            0 => Ok(()),
            -1 => Err(io::Error::new(ErrorKind::InvalidInput, "invalid baudrate")),
            -2 => Err(io::Error::new(ErrorKind::Other, "setting baudrate failed")),
            -3 => Err(io::Error::new(ErrorKind::Other, "USB device unavailable")),
            _ => Err(io::Error::new(ErrorKind::Other, "unknown set baudrate error")),
        }
    }

    pub fn set_bitmode(&mut self, bitmask: u8, mode: MpsseMode) -> io::Result<()> {
        let mode: ffi::ftdi_mpsse_mode = mode.into();
        let result = unsafe { ffi::ftdi_set_bitmode(&mut self.native, bitmask, mode as u8) };
        match result {
            0 => Ok(()),
            -1 => Err(io::Error::new(ErrorKind::Other, "can't enable bitbang mode")),
            -2 => Err(io::Error::new(ErrorKind::Other, "USB device unavailable")),
            _ => Err(io::Error::new(ErrorKind::Other, "unknown set bitmode error")),
        }
    }

    pub fn disable_bitbang(&mut self) -> io::Result<()> {
        let result = unsafe { ffi::ftdi_disable_bitbang(&mut self.native) };
        match result {
            0 => Ok(()),
            -1 => Err(io::Error::new(ErrorKind::Other, "can't disable bitbang mode")),
            -2 => Err(io::Error::new(ErrorKind::Other, "USB device unavailable")),
            _ => Err(io::Error::new(ErrorKind::Other, "unknown disable bitbang error")),
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
        let result = unsafe { ffi::ftdi_write_data(&mut self.native, buf.as_ptr(), len) };
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


#[cfg(test)]
mod test {
    #[test]
    fn it_works() {}
}
