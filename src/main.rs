
use std::{str::FromStr};

use rusb::{
    Context
};

mod usb;
use usb::{ open_device, read_device};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        println!("usage: read_device <vendor-id-in-base-10> <product-id-in-base-10>");
        return;
    }

    
    rusb::set_log_level(rusb::LogLevel::Info);

    let version = rusb::version();

    println!(
        "libusb v{}.{}.{}.{}{}",
        version.major(),
        version.minor(),
        version.micro(),
        version.nano(),
        version.rc().unwrap_or("")
    );
    println!("has capability? {}", rusb::has_capability());
    println!("has hotplug? {}", rusb::has_hotplug());
    println!("has HID access? {}", rusb::has_hid_access());
    println!(
        "supports detach kernel driver? {}",
        rusb::supports_detach_kernel_driver()
    );


    let vid: u16 = FromStr::from_str(args[1].as_ref()).unwrap();
    let pid: u16 = FromStr::from_str(args[2].as_ref()).unwrap();

    match Context::new() {
        Ok(mut context) => match open_device(&mut context, vid, pid) {                    
            Some((mut device, device_desc, mut handle)) => {
                read_device(&mut device, &device_desc, &mut handle).unwrap()
            }
            None => println!("could not find device {:04x}:{:04x}", vid, pid),
        },
        Err(e) => panic!("could not initialize libusb: {}", e),
    }
}