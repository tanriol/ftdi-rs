//! An incomplete Rust wrapper over the `libftdi1` library for working with FTDI devices
//!
//! Note: the library interface is *definitely* unstable for now

use ftdi_mpsse::MpsseCmdBuilder;
use ftdi_mpsse::MpsseCmdExecutor;
use ftdi_mpsse::MpsseSettings;

use libftdi1_sys as ffi;

use std::convert::TryFrom;
use std::convert::TryInto;
use std::io::{self, Read, Write};

pub mod error;
mod opener;

pub use error::{Error, Result};
#[cfg(feature = "libusb1-sys")]
pub use opener::find_by_raw_libusb_device;
pub use opener::{find_by_bus_address, find_by_vid_pid, Opener};

use error::libftdi_to_io;
use error::libusb_to_io;

/// The target interface
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Parity {
    None,
    Odd,
    Even,
    Mark,
    Space,
}

impl Into<ffi::ftdi_parity_type> for Parity {
    fn into(self) -> ffi::ftdi_parity_type {
        match self {
            Parity::None => ffi::ftdi_parity_type::NONE,
            Parity::Odd => ffi::ftdi_parity_type::ODD,
            Parity::Even => ffi::ftdi_parity_type::EVEN,
            Parity::Mark => ffi::ftdi_parity_type::MARK,
            Parity::Space => ffi::ftdi_parity_type::SPACE,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Bits {
    Seven,
    Eight,
}

impl Into<ffi::ftdi_bits_type> for Bits {
    fn into(self) -> ffi::ftdi_bits_type {
        match self {
            Bits::Seven => ffi::ftdi_bits_type::BITS_7,
            Bits::Eight => ffi::ftdi_bits_type::BITS_8,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StopBits {
    One,
    OneHalf,
    Two,
}

impl Into<ffi::ftdi_stopbits_type> for StopBits {
    fn into(self) -> ffi::ftdi_stopbits_type {
        match self {
            StopBits::One => ffi::ftdi_stopbits_type::STOP_BIT_1,
            StopBits::OneHalf => ffi::ftdi_stopbits_type::STOP_BIT_15,
            StopBits::Two => ffi::ftdi_stopbits_type::STOP_BIT_2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FlowControl {
    Disabled,
    RtsCts,
    DtrDsr,
    XonXoff,
}

impl FlowControl {
    pub fn to_ffi(self) -> i32 {
        match self {
            FlowControl::Disabled => ffi::SIO_XON_XOFF_HS,
            FlowControl::RtsCts => ffi::SIO_RTS_CTS_HS,
            FlowControl::DtrDsr => ffi::SIO_DTR_DSR_HS,
            FlowControl::XonXoff => ffi::SIO_XON_XOFF_HS,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BitMode {
    Reset,
    Bitbang,
    Mpsse,
    SyncBB,
    Mcu,
    Opto,
    CBus,
    Syncff,
    Ft1284,
}

impl BitMode {
    pub fn to_ffi(self) -> ffi::ftdi_mpsse_mode {
        match self {
            BitMode::Reset => ffi::ftdi_mpsse_mode::BITMODE_RESET,
            BitMode::Bitbang => ffi::ftdi_mpsse_mode::BITMODE_BITBANG,
            BitMode::Mpsse => ffi::ftdi_mpsse_mode::BITMODE_MPSSE,
            BitMode::SyncBB => ffi::ftdi_mpsse_mode::BITMODE_SYNCBB,
            BitMode::Mcu => ffi::ftdi_mpsse_mode::BITMODE_MCU,
            BitMode::Opto => ffi::ftdi_mpsse_mode::BITMODE_OPTO,
            BitMode::CBus => ffi::ftdi_mpsse_mode::BITMODE_CBUS,
            BitMode::Syncff => ffi::ftdi_mpsse_mode::BITMODE_SYNCFF,
            BitMode::Ft1284 => ffi::ftdi_mpsse_mode::BITMODE_FT1284,
        }
    }
}

pub struct Device {
    context: *mut ffi::ftdi_context,
}

impl Device {
    pub fn set_baud_rate(&mut self, rate: u32) -> Result<()> {
        let rate = rate.try_into().expect("baud rate should fit in an i32");
        let result = unsafe { ffi::ftdi_set_baudrate(self.context, rate) };
        match result {
            0 => Ok(()),
            -1 => Err(Error::InvalidInput("unsupported baudrate")),
            -2 => Err(Error::RequestFailed),
            -3 => unreachable!("uninitialized context"),
            _ => Err(Error::unknown(self.context)),
        }
    }

    pub fn configure(&mut self, bits: Bits, stop_bits: StopBits, parity: Parity) -> Result<()> {
        let result = unsafe {
            ffi::ftdi_set_line_property(self.context, bits.into(), stop_bits.into(), parity.into())
        };
        match result {
            0 => Ok(()),
            -1 => Err(Error::RequestFailed),
            -2 => unreachable!("uninitialized context"),
            _ => Err(Error::unknown(self.context)),
        }
    }

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

    pub fn usb_purge_tx_buffer(&mut self) -> Result<()> {
        let result = unsafe { ffi::ftdi_usb_purge_tx_buffer(self.context) };
        match result {
            0 => Ok(()),
            -1 => Err(Error::RequestFailed),
            -2 => unreachable!("uninitialized context"),
            _ => Err(Error::unknown(self.context)),
        }
    }

    pub fn usb_set_event_char(&mut self, value: Option<u8>) -> Result<()> {
        let result = if let Some(v) = value {
            unsafe { ffi::ftdi_set_event_char(self.context, v, 1) }
        } else {
            unsafe { ffi::ftdi_set_event_char(self.context, 0, 0) }
        };

        match result {
            0 => Ok(()),
            -1 => Err(Error::RequestFailed),
            -2 => unreachable!("uninitialized context"),
            _ => Err(Error::unknown(self.context)),
        }
    }

    pub fn usb_set_error_char(&mut self, value: Option<u8>) -> Result<()> {
        let result = if let Some(v) = value {
            unsafe { ffi::ftdi_set_error_char(self.context, v, 1) }
        } else {
            unsafe { ffi::ftdi_set_error_char(self.context, 0, 0) }
        };

        match result {
            0 => Ok(()),
            -1 => Err(Error::RequestFailed),
            -2 => unreachable!("uninitialized context"),
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

    pub fn set_flow_control(&mut self, flowctrl: FlowControl) -> Result<()> {
        let result = unsafe { ffi::ftdi_setflowctrl(self.context, flowctrl.to_ffi()) };
        match result {
            0 => Ok(()),
            -1 => Err(Error::RequestFailed),
            -2 => unreachable!("uninitialized context"),
            _ => Err(Error::unknown(self.context)),
        }
    }

    pub fn set_bitmode(&mut self, output_mask: u8, mode: BitMode) -> Result<()> {
        let mode = mode.to_ffi().0.try_into().unwrap();
        let result = unsafe { ffi::ftdi_set_bitmode(self.context, output_mask, mode) };
        match result {
            0 => Ok(()),
            -1 => Err(Error::RequestFailed),
            -2 => unreachable!("uninitialized context"),
            _ => Err(Error::unknown(self.context)),
        }
    }

    pub fn libftdi_context(&mut self) -> *mut ffi::ftdi_context {
        self.context
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        let result = unsafe { ffi::ftdi_usb_close(self.context) };
        match result {
            0 => {}
            -1 => { /* TODO emit warning ("usb_release failed") */ }
            -3 => unreachable!("uninitialized context"),
            _ => panic!("undocumented ftdi_usb_close return value"),
        };
        unsafe {
            ffi::ftdi_free(self.context);
        }
    }
}

impl Read for Device {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = buf.len().try_into().unwrap_or(std::i32::MAX);
        let result = unsafe { ffi::ftdi_read_data(self.context, buf.as_mut_ptr(), len) };
        match result {
            count if count >= 0 => Ok(count as usize),
            -666 => unreachable!("uninitialized context"),
            err => Err(libusb_to_io(err)),
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
            err => Err(libusb_to_io(err)),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Device {
    pub fn set_mpsse_clock(&mut self, freq: u32) -> std::result::Result<(), io::Error> {
        const MAX: u32 = 30_000_000;
        const MIN: u32 = 92;

        assert!(
            freq >= MIN,
            "frequency of {} exceeds minimum of {}",
            freq,
            MIN
        );
        assert!(
            freq <= MAX,
            "frequency of {} exceeds maximum of {}",
            freq,
            MAX
        );

        let (divisor, clkdiv) = if freq <= 6_000_000 {
            (6_000_000 / freq - 1, Some(true))
        } else {
            (30_000_000 / freq - 1, Some(false))
        };

        self.write_all(MpsseCmdBuilder::new().set_clock(divisor, clkdiv).as_slice())?;

        Ok(())
    }
}

impl MpsseCmdExecutor for Device {
    type Error = io::Error;

    /// Initialize the MPSSE controller.
    ///
    /// According to AN135 [FTDI MPSSE basics], this method does the following:
    /// 1. Optionally resets the peripheral side of FTDI port.
    /// 2. Configures the maximum USB transfer sizes.
    /// 3. Disables any event or error special characters.
    /// 4. Configures the read and write timeouts (not yet implemented).
    /// 5. Configures the latency timer to wait before sending an incomplete USB packet
    ///    from the peripheral back to the host.
    /// 6. Configures for RTS/CTS flow control to ensure that the driver will not issue
    ///    IN requests if the buffer is unable to accept data.
    /// 7. Resets and then enables the MPSSE controller
    /// 8. Optionally configures the MPSSE clock frequency.
    ///
    /// [FTDI MPSSE Basics]: https://www.ftdichip.com/Support/Documents/AppNotes/AN_135_MPSSE_Basics.pdf
    fn init(&mut self, settings: &MpsseSettings) -> std::result::Result<(), io::Error> {
        let millis = u8::try_from(settings.latency_timer.as_millis())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        if settings.reset {
            self.usb_reset().map_err(libftdi_to_io)?;
        }

        self.usb_purge_buffers().map_err(libftdi_to_io)?;
        self.set_write_chunksize(settings.in_transfer_size);
        self.set_read_chunksize(settings.in_transfer_size);
        self.set_latency_timer(millis).map_err(libftdi_to_io)?;
        self.usb_set_event_char(None).map_err(libftdi_to_io)?;
        self.usb_set_error_char(None).map_err(libftdi_to_io)?;
        self.set_flow_control(FlowControl::RtsCts)
            .map_err(libftdi_to_io)?;
        self.set_bitmode(0, BitMode::Reset).map_err(libftdi_to_io)?;
        self.set_bitmode(settings.mask, BitMode::Mpsse)
            .map_err(libftdi_to_io)?;

        if let Some(frequency) = settings.clock_frequency {
            self.set_mpsse_clock(frequency)?;
        }

        Ok(())
    }

    /// Write the MPSSE command to the device.
    ///
    /// Workaround: perform Tx flush before sending data.
    /// Otherwise several first readings from FTDI chip in the exchange sequence may be broken.
    /// E.g. reading during the first exchange returns nothing, but reading during the second
    /// exchange returns both the first and second replies. So far it is not clear how to get
    /// rid of this workaround. Note that such a workaround is not needed when FTDI proprietary
    /// libftd2xx library is used.
    fn send(&mut self, data: &[u8]) -> std::result::Result<(), io::Error> {
        self.usb_purge_tx_buffer()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        self.write_all(data)
    }

    /// Read the MPSSE response from the device.
    fn recv(&mut self, data: &mut [u8]) -> std::result::Result<(), io::Error> {
        self.read_exact(data)
    }
}
