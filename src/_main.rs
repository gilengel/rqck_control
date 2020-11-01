use rusb::{Context, Device, DeviceHandle, Result, UsbContext, DeviceDescriptor, Language, TransferType, Direction, request_type, RequestType, Recipient, supports_detach_kernel_driver};
use std::time::Duration;
use std::{str::FromStr};


struct DeviceId {
    pub vendor_id: u16,
    pub product_id: u16
}

#[derive(Debug)]
struct Endpoint {
    config: u8,
    iface: u8,
    setting: u8,
    address: u8,
}

// returns all readable endpoints for given usb device and descriptor
fn find_readable_endpoints<T: UsbContext>(device: &mut Device<T>) -> Result<Vec<Endpoint>> {
    let device_desc = device.device_descriptor()?;
    
    let mut endpoints = vec![];
    for n in 0..device_desc.num_configurations() {
        let config_desc = match device.config_descriptor(n) {
            Ok(c) => c,
            Err(_) => continue,
        };
        
        //println!("{:#?}", config_desc);
        for interface in config_desc.interfaces() {
            for interface_desc in interface.descriptors() {
                
                //println!("{:#?}", interface_desc);
                for endpoint_desc in interface_desc.endpoint_descriptors() {
                    
                    //println!("{:#?}", endpoint_desc);
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

    Ok(endpoints)
}

fn configure_endpoint<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    endpoint: &Endpoint,
) -> Result<()> {
    handle.set_active_configuration(endpoint.config)?;
    handle.claim_interface(endpoint.iface)?;
    handle.set_alternate_setting(endpoint.iface, endpoint.setting)
}

fn set_report<T: UsbContext>(handle: &mut DeviceHandle<T>) -> Result<usize> {
    let timeout = Duration::from_secs(1);

    // values are picked directly from the captured packet
    const REQUEST_TYPE: u8 = 0x21;
    const REQUEST: u8 = 0x09; // SET_REPORT (0x09)
    const VALUE: u16 = 0x0200;
    const INDEX: u16 = 0x0000;
    const DATA: [u8; 71] = [    
        0x09, 0x00, 0x02, 0x00, 0x00, 0x40, 0x00, 0x0d, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
    ];

    handle.write_control(REQUEST_TYPE, REQUEST, VALUE, INDEX, &DATA, timeout)
}

fn set_report2<T: UsbContext>(handle: &mut DeviceHandle<T>) -> Result<usize> {
    let timeout = Duration::from_secs(1);

    // values are picked directly from the captured packet
    const REQUEST_TYPE: u8 = 0x21;
    const REQUEST: u8 = 0x09; // SET_REPORT (0x09)
    const VALUE: u16 = 0x0200;
    const INDEX: u16 = 0x0000;
    const DATA: [u8; 71] = [    
        0x09, 0x00, 0x02, 0x00, 0x00, 0x40, 0x00, 0x0c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
    ];

    handle.write_control(REQUEST_TYPE, REQUEST, VALUE, INDEX, &DATA, timeout)
}


fn read_interrupt<T: UsbContext>(handle: &mut DeviceHandle<T>, address: u8) -> Result<Vec<u8>> {
    let timeout = Duration::from_secs(1);
    let mut buf = [0u8; 64];

    handle
        .read_interrupt(address, &mut buf, timeout)
        .map(|_| buf.to_vec())
}

fn read_report<T: UsbContext>(handle: &mut DeviceHandle<T>) -> Result<Vec<u8>> {
    let timeout = Duration::from_secs(1);
    let mut buf = [0u8; 28];

  /*  
    LIBUSB_REQUEST_GET_DESCRIPTOR,
    u16::from(LIBUSB_DT_STRING) << 8,
    0,
*/

    handle
    .read_control(request_type(Direction::Out, RequestType::Standard, Recipient::Endpoint),0x09, 0x0200, 0x0000, &mut buf, timeout)    
    //.read_control(0x80, 0x09, 0x0200, 0x0000, &mut buf, timeout)    
    //.read_control(address, &mut buf, timeout)
    //.read_interrupt(address, &mut buf, timeout)
        .map(|_| buf.to_vec())
}

fn find_readable_endpoint<T: UsbContext>(
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
                    
                    println!("{:?}", endpoint_desc);
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

fn main() -> Result<()> {
    println!("Support for kernel detach {}", supports_detach_kernel_driver());

    //search_device(&"SteelSeries".to_string(), &"Steelseries QCK Prism Cloth".to_string());
    let mut context = Context::new()?;
    let (mut device, device_desc, mut handle) =
        //open_device(&mut context, 4152, 5389).expect("Failed to open USB device"); windows
        open_device(&mut context, 0x1038, 0x150d).expect("Failed to open USB device"); //linux

        if let Some(endpoint) = find_readable_endpoint(&mut device, &device_desc, TransferType::Interrupt) {
            println!("Found the following endpoints: {:?}", endpoint.len());
            println!("===============================");
            for e in &endpoint {
                println!("{:?}", e);
            }
            println!("===============================");


            let endpoint = &endpoint[0];
            let has_kernel_driver = match handle.kernel_driver_active(endpoint.iface) {
                Ok(true) => {
                    match handle.detach_kernel_driver(endpoint.iface) {
                        Ok(_) => println!("Default driver detached"),
                        Err(e) => println!("Default driver was not detached, error {}", e)
                    }
                    
                    true
                }
                Err(e) => {
                    println!("Määäääääh");

                    false
                }
                _ => false,
            };
        
            println!(" - kernel driver? {}", has_kernel_driver);

            let endpoint_config_result = configure_endpoint(&mut handle, &endpoint); 

            match set_report(&mut handle) {
                Ok(_) => println!("Report set :)"),
                Err(e) => println!("Report not set :( {}", e)
            }
        
            
            match endpoint_config_result {
                Ok(_) => {
                    let mut buf = [0; 256];
                    let timeout = Duration::from_secs(1);
        
                    match handle.read_interrupt(endpoint.address, &mut buf, timeout) {
                        Ok(len) => {
                            println!(" - read: {:?}", &buf[..len]);
                        }
                        Err(err) => println!("could not read from endpoint: {}", err),
                    }
                }
                Err(err) => println!("could not configure endpoint: {}", err),
            }
            
        
            handle.release_interface(endpoint.iface)?;
            if has_kernel_driver {
                handle.attach_kernel_driver(endpoint.iface).ok();
            }
        }

        

        /*
        
        set_report2(&mut handle)?;
        */

    
    Ok(())
    /*
    for device in rusb::devices().unwrap().iter() {
        let device_desc = device.device_descriptor().unwrap();

        println!("Bus {:03} Device {:03} ID {:04x}:{:04x}",
            device.bus_number(),
            device.address(),
            device_desc.vendor_id(),
            device_desc.product_id());
    }
    */
}


// device uid pid are picked directly form `lsusb` result
const VID: u16 = 0x1038;
const PID: u16 = 0x150d;

/*
fn main() -> Result<()> {
    let mut context = Context::new()?;
    let (mut device, mut handle) =
        open_device(&mut context, VID, PID).expect("Failed to open USB device");

    print_device_info(&mut handle)?;
    Ok(())
}
*/

fn open_device<T: UsbContext>(
    context: &mut T,
    vid: u16,
    pid: u16,
) -> Option<(Device<T>, DeviceDescriptor, DeviceHandle<T>)> {
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

fn print_device_info<T: UsbContext>(handle: &mut DeviceHandle<T>) -> Result<()> {
    let device_desc = handle.device().device_descriptor()?;
    let timeout = Duration::from_secs(1);
    let languages = handle.read_languages(timeout)?;

    println!("Active configuration: {}", handle.active_configuration()?);

    if !languages.is_empty() {
        let language = languages[0];
        println!("Language: {:?}", language);

        println!(
            "Manufacturer: {}",
            handle
                .read_manufacturer_string(language, &device_desc, timeout)
                .unwrap_or("Not Found".to_string())
        );
        println!(
            "Product: {}",
            handle
                .read_product_string(language, &device_desc, timeout)
                .unwrap_or("Not Found".to_string())
        );
        println!(
            "Serial Number: {}",
            handle
                .read_serial_number_string(language, &device_desc, timeout)
                .unwrap_or("Not Found".to_string())
        );
    }
    Ok(())
}

fn read_device_description_and_languages<T: UsbContext>(handle: &mut DeviceHandle<T>) -> Result<(DeviceDescriptor, Vec<Language>)> {
    let device_desc = handle.device().device_descriptor()?;
    let timeout = Duration::from_secs(1);
    let languages = handle.read_languages(timeout)?;

    Ok((device_desc, languages))
}

fn is_correct_device<T: UsbContext>(handle: &mut DeviceHandle<T>, manufacturer: &str, product: &str) -> bool {
    let device_desc_and_language =  read_device_description_and_languages(handle);
    let timeout = Duration::from_secs(1);

    if device_desc_and_language.is_ok() {
        let device_desc_and_language = device_desc_and_language.unwrap();
        if !device_desc_and_language.1.is_empty() {
            let language = device_desc_and_language.1[0];
    
            if manufacturer != handle
            .read_manufacturer_string(language, &device_desc_and_language.0, timeout)
            .unwrap_or("Not Found".to_string()) {
                return false;
            }
    
            if product != handle
            .read_product_string(language, &device_desc_and_language.0, timeout)
            .unwrap_or("Not Found".to_string()) {
                return false;
            }
        
            return true;
        }    
    }

    false
}