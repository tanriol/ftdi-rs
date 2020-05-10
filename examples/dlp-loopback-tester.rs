//! A simple test script for testing the loopback features
//! of the FTDI DLP-HS-FPGA3's default FPGA firmware.
//! Mainly used as a compile test

extern crate ftdi;

use std::io::{Read, Write};

fn main() {
    println!("Starting tester...");
    let device = ftdi::find_by_vid_pid(0x0403, 0x6010)
        .interface(ftdi::Interface::A)
        .open();

    if let Ok(mut device) = device {
        println!("Device found and opened");
        device.usb_reset().unwrap();
        device.usb_purge_buffers().unwrap();
        device.set_latency_timer(2).unwrap();

        // Junk test
        let mut junk = vec![];
        device.read_to_end(&mut junk).unwrap();
        if junk.len() > 0 {
            println!("Junk in line: {:?}", junk);
        }

        // Ping test
        device.write_all(&vec![0x00]).unwrap();
        let mut reply = vec![];
        device.read_to_end(&mut reply).unwrap();
        if reply != vec![0x56] {
            println!("Wrong ping reply {:?} (expected {:?}", reply, vec![0x56]);
        }

        for num in 0u16..256 {
            let num = num as u8;

            // Loopback test
            device.write_all(&vec![0x20, num]).unwrap();
            let mut reply = vec![];
            device.read_to_end(&mut reply).unwrap();
            if reply != vec![num] {
                println!("Wrong loopback reply {:?} (expected {:?}", reply, vec![num]);
            }

            // Complement loopback test
            device.write_all(&vec![0x21, num]).unwrap();
            let mut reply = vec![];
            device.read_to_end(&mut reply).unwrap();
            let complement = 255 - num;
            if reply != vec![complement] {
                println!(
                    "Wrong complement reply {:?} (expected {:?}",
                    reply,
                    vec![complement]
                );
            }
        }
        println!("Testing finished");
    } else {
        println!("Cannot find/open device, runtime tests are NOP");
    }
}
