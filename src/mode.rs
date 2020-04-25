use std::convert::TryFrom;

use super::{Device, Error, Result};

use super::bitbang;
use super::ffi;

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

        u8::try_from(mode.0).unwrap_or(0)
    }
}

pub(crate) trait BitBang
where
    Self: std::marker::Sized,
{
    fn new(dev: Device, bitmask: u8) -> Result<Self>;
    fn disable(self) -> Result<Device>;
}

impl BitBang for bitbang::BitBang {
    fn new(device: Device, bitmask: u8) -> Result<Self> {
        let result =
            unsafe { ffi::ftdi_set_bitmode(device.context, bitmask, BitMode::BITBANG.into()) };

        match result {
            0 => {
                let bb = bitbang::BitBang { device };

                Ok(bb)
            }
            -1 => Err(Error::RequestFailed),
            -2 => unreachable!("uninitialized context"),
            _ => Err(Error::unknown(device.context)),
        }
    }

    fn disable(mut self) -> Result<Device> {
        let result = unsafe { ffi::ftdi_disable_bitbang(self.device.context) };

        match result {
            0 => {
                let context = std::mem::replace(&mut self.device.context, std::ptr::null_mut());

                let device = Device { context };

                Ok(device)
            }
            -1 => Err(Error::RequestFailed),
            -2 => unreachable!("uninitialized context"),
            _ => Err(Error::unknown(self.device.context)),
        }
    }
}
