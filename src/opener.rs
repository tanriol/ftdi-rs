use std::ffi::CString;

use super::{ffi, Device, Error, Interface, Result};

pub trait Target {
    fn open_in_context(self, context: *mut ffi::ftdi_context) -> Result<()>;
}

pub struct BusAddress {
    bus: u8,
    address: u8,
}

impl Target for BusAddress {
    fn open_in_context(self, context: *mut ffi::ftdi_context) -> Result<()> {
        let result = unsafe { ffi::ftdi_usb_open_bus_addr(context, self.bus, self.address) };
        match result {
            0 => Ok(()),
            -1 => Err(Error::EnumerationFailed), // usb_find_busses() failed
            -2 => Err(Error::EnumerationFailed), // usb_find_devices() failed
            -3 => Err(Error::DeviceNotFound),    // usb device not found
            -4 => Err(Error::AccessFailed),      // unable to open device
            -5 => Err(Error::ClaimFailed),       // unable to claim device
            -6 => Err(Error::RequestFailed),     // reset failed
            -7 => Err(Error::RequestFailed),     // set baudrate failed
            -8 => Err(Error::EnumerationFailed), // get product description failed
            -9 => Err(Error::EnumerationFailed), // get serial number failed
            -10 => Err(Error::unknown(context)), // unable to close device
            -11 => unreachable!("uninitialized context"), // ftdi context invalid
            -12 => Err(Error::EnumerationFailed), // libusb_get_device_list() failed
            _ => Err(Error::unknown(context)),
        }
    }
}

pub struct UsbProperties {
    vid: u16,
    pid: u16,
    description: Option<CString>,
    serial: Option<CString>,
    index: Option<u32>,
}

impl Target for UsbProperties {
    fn open_in_context(self, context: *mut ffi::ftdi_context) -> Result<()> {
        let description = self
            .description
            .map(|s| s.as_ptr())
            .unwrap_or(std::ptr::null());
        let serial = self.serial.map(|s| s.as_ptr()).unwrap_or(std::ptr::null());
        let index = self.index.unwrap_or(0).into();
        let result = unsafe {
            ffi::ftdi_usb_open_desc_index(
                context,
                self.vid.into(),
                self.pid.into(),
                description,
                serial,
                index,
            )
        };
        match result {
            0 => Ok(()),
            -1 => Err(Error::EnumerationFailed), // usb_find_busses() failed
            -2 => Err(Error::EnumerationFailed), // usb_find_devices() failed
            -3 => Err(Error::DeviceNotFound),    // usb device not found
            -4 => Err(Error::AccessFailed),      // unable to open device
            -5 => Err(Error::ClaimFailed),       // unable to claim device
            -6 => Err(Error::RequestFailed),     // reset failed
            -7 => Err(Error::RequestFailed),     // set baudrate failed
            -8 => Err(Error::EnumerationFailed), // get product description failed
            -9 => Err(Error::EnumerationFailed), // get serial number failed
            -10 => Err(Error::unknown(context)), // unable to close device
            -11 => unreachable!("uninitialized context"), // ftdi context invalid
            -12 => Err(Error::EnumerationFailed), // libusb_get_device_list() failed
            _ => Err(Error::unknown(context)),
        }
    }
}

pub struct Opener<T: Target> {
    target: T,
    interface: Option<Interface>,
}

impl<T: Target> Opener<T> {
    fn new(target: T) -> Self {
        Self {
            target,
            interface: None,
        }
    }

    pub fn interface(mut self, interface: Interface) -> Self {
        assert!(self.interface.is_none(), "interface already set");
        self.interface = Some(interface);
        self
    }

    pub fn open(self) -> Result<Device> {
        let context = unsafe { ffi::ftdi_new() };

        if context.is_null() {
            return Err(Error::AllocationFailed);
        }

        if let Some(interface) = self.interface {
            let result = unsafe { ffi::ftdi_set_interface(context, interface.into()) };
            match result {
                0 => Ok(()),
                -1 => unreachable!("unknown interface from ftdi.h"),
                -2 => unreachable!("missing context"),
                -3 => unreachable!("device already opened in Builder"),
                _ => Err(Error::unknown(context)),
            }?;
        }

        self.target.open_in_context(context)?;

        Ok(Device { context })
    }
}

impl Opener<UsbProperties> {
    pub fn description(mut self, description: &str) -> Self {
        assert!(self.target.description.is_none(), "description already set");
        self.target.description =
            Some(CString::new(description).expect("serial should not contain NUL"));
        self
    }

    pub fn serial(mut self, serial: &str) -> Self {
        assert!(self.target.serial.is_none(), "serial already set");
        self.target.serial = Some(CString::new(serial).expect("serial should not contain NUL"));
        self
    }

    pub fn nth(mut self, index: u32) -> Self {
        assert!(self.target.index.is_none(), "index already set");
        self.target.index = Some(index);
        self
    }
}

pub fn find_by_vid_pid(vid: u16, pid: u16) -> Opener<UsbProperties> {
    Opener::new(UsbProperties {
        vid,
        pid,
        description: None,
        serial: None,
        index: None,
    })
}

pub fn find_by_bus_address(bus: u8, address: u8) -> Opener<BusAddress> {
    Opener::new(BusAddress { bus, address })
}

#[cfg(feature = "libusb1-sys")]
use ffi::libusb1_sys::libusb_device;

#[cfg(feature = "libusb1-sys")]
pub struct LibusbDevice {
    device: *mut libusb_device,
}

#[cfg(feature = "libusb1-sys")]
impl Target for LibusbDevice {
    fn open_in_context(self, context: *mut ffi::ftdi_context) -> Result<()> {
        let result = unsafe { ffi::ftdi_usb_open_dev(context, self.device) };
        match result {
            0 => Ok(()),
            -3 => Err(Error::AccessFailed), // unable to config device
            -4 => Err(Error::AccessFailed), // unable to open device
            -5 => Err(Error::ClaimFailed),  // unable to claim device
            -6 => Err(Error::RequestFailed), // reset failed
            -7 => Err(Error::RequestFailed), // set baudrate failed
            -8 => unreachable!("uninitialized context"), // ftdi context invalid
            -9 => Err(Error::AccessFailed), // libusb_get_device_descriptor() failed
            -10 => Err(Error::AccessFailed), // libusb_get_config_descriptor() failed
            -11 => Err(Error::AccessFailed), // libusb_detach_kernel_driver() failed
            -12 => Err(Error::AccessFailed), // libusb_get_configuration() failed
            _ => Err(Error::unknown(context)),
        }
    }
}

#[cfg(feature = "libusb1-sys")]
pub unsafe fn find_by_raw_libusb_device(device: *mut libusb_device) -> Opener<LibusbDevice> {
    Opener::new(LibusbDevice { device })
}
