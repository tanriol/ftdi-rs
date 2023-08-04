//! A test script, puts an e.g. FT232H into synchronous FIFO mode and reads data from it.
//! You will likely need to set (245) FIFO mode in eeprom, first; see FTDI's programmer.
//! You MAY also need to `sudo rmmod ftdi_sio`, try that if it can't find/open the device.
//! Assumes data is being streamed; won't block.

extern crate ftdi;

use std::{io::Read, time::Instant, convert::TryInto, convert::TryFrom};
use std::cmp::min;

const RX_BUF_SIZE: usize = 0x100000;
const MAX_PRINT_SIZE: usize = 0x10;
// const MAX_PRINT_SIZE: usize = 0x1000000;
const ITER: i32 = 0x01;

fn main() {
    println!("Starting tester...");
    let device = ftdi::find_by_vid_pid(0x0403, 0x6014) // FT232H
        .interface(ftdi::Interface::A)
        .open();

    if let Ok(mut device) = device {
        println!("Device found and opened");
        device.usb_reset().unwrap();
        device.usb_purge_buffers().unwrap();
        device.set_latency_timer(2).unwrap();

        // Missing: set_usb_parameters
        device.usb_set_event_char(None).unwrap();
        device.usb_set_error_char(None).unwrap();
        // Missing: set_timeouts
        //ft.set_latency_timer(Duration::from_millis(16))?;
        //ft.set_flow_control_rts_cts()?;
        device.set_bitmode(0x00, ftdi::BitMode::Reset).unwrap();
        device.set_bitmode(0x00, ftdi::BitMode::Syncff).unwrap(); // Synchronous FIFO

        let mut rx_buf: Vec<u8> = vec![0; RX_BUF_SIZE];

        let mut total_us: u128 = 0;
        let mut total_bytes: u128 = 0;
        for _ in 0..ITER {
            print!("...");
            let now = Instant::now();
            let ret = device.read(&mut rx_buf).unwrap();
            let t: u128 = now.elapsed().as_micros();
            let z: u128 = (ret * 1000000).try_into().unwrap();
            total_us += t;
            total_bytes += u128::try_from(ret).unwrap();
    
            println!("{ret} @ {} = {} B/s", t, z/t);

            let n = min(ret, MAX_PRINT_SIZE);
            print!("rx: ");
            if n < ret {
                for i in 0..n {
                    print!("{:02X}", rx_buf[i]);
                }
                print!("...");
                for i in (ret-n)..ret {
                    print!("{:02X}", rx_buf[i]);
                }
            } else {
                for i in 0..n {
                    print!("{:02X}", rx_buf[i]);
                }
            }

            println!();
            println!();
        }
    
        let t: u128 = total_us;
        let z: u128 = (total_bytes * 1000000).try_into().unwrap();
        println!("total {total_bytes} @ {} = {} B/s", t, z/t); // Doesn't account for print time, buffering means this may be incorrect
    
        println!();
        
        println!("Testing finished");
    } else {
        println!("Cannot find/open device, runtime tests are NOP");
    }
}
