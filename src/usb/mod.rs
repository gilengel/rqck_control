use rusb::{
    Context, Device, DeviceDescriptor, DeviceHandle, Direction, Result, TransferType, UsbContext, 
};

use termion::color;


use std::{
    time::Duration
};

#[derive(Debug)]
pub struct Endpoint {
    config: u8,
    iface: u8,
    setting: u8,
    address: u8,
}

/// The presets correspond to the intensity options provided in the Windows control
/// by steelseries.
pub enum IntensityPresets {
    Off = 0,
    Dark = 85,
    Bright = 170,
    Brightest = 255
}

pub fn open_device<T: UsbContext>(
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

pub fn read_device<T: UsbContext>(
    device: &mut Device<T>,
    device_desc: &DeviceDescriptor,
    handle: &mut DeviceHandle<T>,
) -> Result<()> {
    //handle.reset()?;

    match handle.set_auto_detach_kernel_driver(false) {
        Ok(_) => println!("[OPTION] auto detach kernel driver disabled"),
        Err(_) => println!("{}[OPTION] auto detach kernel driver is not supported and therefore not disabled", color::Fg(color::LightRed))
    }
    

    //let timeout = Duration::from_secs(1);
    //let languages = handle.read_languages(timeout)?;

    println!("Active configuration: {}", handle.active_configuration()?);
    /*
    println!("Languages: {:?}", languages);

    
    if languages.len() > 0 {
        let language = languages[0];

        println!(
            "Manufacturer: {:?}",
            handle
                .read_manufacturer_string(language, device_desc, timeout)
                .ok()
        );
        println!(
            "Product: {:?}",
            handle
                .read_product_string(language, device_desc, timeout)
                .ok()
        );
        println!(
            "Serial Number: {:?}",
            handle
                .read_serial_number_string(language, device_desc, timeout)
                .ok()
        );
    }
    */

    match find_readable_endpoint(device, device_desc, TransferType::Interrupt) {
        Some(endpoints) => {


            let mut endpoints_have_kernel_driver: Vec<(&Endpoint, bool)> = Vec::new();
            for endpoint in &endpoints {
                let has_kernel_driver = match handle.kernel_driver_active(endpoint.iface) {
                    Ok(true) => {
                        match handle.detach_kernel_driver(endpoint.iface) {
                            Ok(_) => println!("Kernel driver successfully detached for endpoint {:?}", endpoint),
                            Err(e) => println!("Error while trying to detach kernel driver: {}", e),
                        }
                        true
                    }
                    _ => false,
                }; 
                
                endpoints_have_kernel_driver.push((endpoint, has_kernel_driver));
            }

            for endpoint in &endpoints {
                configure_endpoint(handle, &endpoint);
            }

            
            match set_intensity(handle, 255) {
                Ok(e) => println!("{}[SET_REPORT] successfull transfered {} bytes", color::Fg(color::White), e),
                Err(e) => println!("{}[SET_REPORT] error:{:?}", color::Fg(color::Red), e),
            }
            

            /*
            match set_report2(handle) {
                Ok(e) => println!("{}[SET_REPORT2] successfully transfered {} bytes", color::Fg(color::White), e),
                Err(e) => println!("{}[SET_REPORT2] error: {:?}", color::Fg(color::Red), e)
            }
            */


            //println!("{:?}", endpoints.first().unwrap()); 
            //read_endpoint(handle,  &endpoints[0]);
            //read_endpoint(handle,  &endpoints[1]);

            for (endpoint, has_kernel_driver) in endpoints_have_kernel_driver {
                match handle.release_interface(endpoint.iface) {
                    Ok(_) => println!("{}Interrupt interface successfully released for endpoint {:?}", color::Fg(color::White), endpoint),
                    Err(e) => println!("{}Error while trying to release interrupt interface: {}", color::Fg(color::Red), e),
                }
                if has_kernel_driver {
                    match handle.attach_kernel_driver(endpoint.iface) {
                        Ok(_) => println!("{}Kernel driver successfully atached for endpoint {:?}", color::Fg(color::White), endpoint),
                        Err(e) => println!("{}Error while trying to atached kernel driver: {}", color::Fg(color::Red), e),
                    }
                }
            }
        },
        None => println!("No readable interrupt endpoint"),
    }

    Ok(())
}

pub fn find_readable_endpoint<T: UsbContext>(
    device: &mut Device<T>,
    device_desc: &DeviceDescriptor,
    transfer_type: TransferType,
) -> Option<Vec<Endpoint>> {
    let mut endpoints = Vec::new();
    for n in 0..device_desc.num_configurations() {
        let config_desc = match device.config_descriptor(n) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for interface in config_desc.interfaces() {
            for interface_desc in interface.descriptors() {
                for endpoint_desc in interface_desc.endpoint_descriptors() {   
                    println!("endpoint_desc {:?}", endpoint_desc);  
                    if endpoint_desc.direction() == Direction::In
                        && endpoint_desc.transfer_type() == transfer_type
                    {
                        endpoints.push(Endpoint {
                            config: config_desc.number(),
                            iface: interface_desc.interface_number(),
                            setting: interface_desc.setting_number(),
                            address: endpoint_desc.address(),
                        });  
                    }
                }
            }
        }

        
    }

    return Some(endpoints);
}

pub fn read_endpoint<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    endpoint: &Endpoint,
) {
    let timeout = Duration::from_secs(5);
    let mut buf = [0u8; 24];
    println!("{}Reading from endpoint: {:?}", color::Fg(color::White), endpoint);

    match handle.read_interrupt(endpoint.address, &mut buf, timeout) {
        Ok(len) => {
            println!(" - read: {:?}", &buf[..len]);
        },
        Err(e) => println!("{}Error while reading from endpoint: {:?}: {}", color::Fg(color::Red), endpoint, e)
        
    }
}

pub fn configure_endpoint<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    endpoint: &Endpoint,
) {   
    
    match handle.set_active_configuration(endpoint.config) {
        Ok(_) => (),
        Err(e) => { println!("{}Error while setting configuration for endpoint {:?}: {}", color::Fg(color::Red), endpoint, e); }
    }
    
    match handle.claim_interface(endpoint.iface) {
        Ok(_) => (),
        Err(e) => { println!("Error while claiming interface for endpoint {:?}: {}",  endpoint, e); }
    }
    
    match handle.set_alternate_setting(endpoint.iface, endpoint.setting){
        Ok(_) => (),
        Err(e) => { println!("Error while set alternate settings for endpoint {:?}: {}",  endpoint, e); }
    }
}

pub fn set_intensity_with_preset<T: UsbContext>(handle: &mut DeviceHandle<T>, intensity: IntensityPresets) -> Result<usize> {
    set_intensity(handle, intensity as u8)
}

pub fn set_intensity<T: UsbContext>(handle: &mut DeviceHandle<T>, intensity: u8) -> Result<usize> {
    let timeout = Duration::from_secs(1);

    let data: [u8; 64] = [    
        0x0c, 0x00, intensity, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
    ];

    handle.write_control(0x21, 0x09, 0x0200, 0x0000, &data, timeout)
}
