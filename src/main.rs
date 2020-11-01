use std::{str::FromStr, time::Duration};

use rusb::{
    Context, Device, DeviceDescriptor, DeviceHandle, Result, UsbContext, 
};


use termion::color;

#[derive(Debug)]
struct Endpoint {
    config: u8,
    iface: u8,
    setting: u8,
    address: u8,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 4 {
        println!("usage: read_device <vendor-id-in-base-10> <product-id-in-base-10> <intensity-in-base-10");
        return;
    }

    let vid: u16 = FromStr::from_str(args[1].as_ref()).unwrap();
    let pid: u16 = FromStr::from_str(args[2].as_ref()).unwrap();
    let intensity: u8 = FromStr::from_str(args[3].as_ref()).unwrap();

    match Context::new() {
        Ok(mut context) => match open_device(&mut context, vid, pid) {                    
            Some((_, _, mut handle)) => {
                match set_intensity(&mut handle, intensity) {
                    Ok(e) => println!("{}[SET_REPORT] successfull transfered {} bytes", color::Fg(color::White), e),
                    Err(e) => println!("{}[SET_REPORT] error:{:?}", color::Fg(color::Red), e),
                } 

            }
            None => println!("could not find device {:04x}:{:04x}", vid, pid),
        },
        Err(e) => panic!("could not initialize libusb: {}", e),
    }
}

fn set_intensity<T: UsbContext>(handle: &mut DeviceHandle<T>, intensity: u8) -> Result<usize> {
    let timeout = Duration::from_secs(1);

    let data: [u8; 64] = [    
        0x0c, 0x00, intensity, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
    ];

    handle.write_control(0x21, 0x09, 0x0200, 0x0000, &data, timeout)
}

fn open_device<T: UsbContext>(
    context: &mut T,
    vid: u16,
    pid: u16,
) -> Option<(Device<T>, DeviceDescriptor, DeviceHandle<T>)> {
    context.set_log_level(rusb::LogLevel::Debug);

    let devices = match context.devices() {
        Ok(d) => d,
        Err(_) => return None,
    };

    for device in devices.iter() {
        let device_desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue,
        };

        if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
            match device.open() {
                Ok(handle) => return Some((device, device_desc, handle)),
                Err(_) => continue,
            }
        }
    }

    None
}