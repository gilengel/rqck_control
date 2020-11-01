use rusb::{Context, Device, DeviceDescriptor, DeviceHandle, Language, Result, UsbContext};
use std::time::Duration;

fn read_device_description_and_languages<T: UsbContext>(handle: &mut DeviceHandle<T>) -> Result<(DeviceDescriptor, Vec<Language>)> {
    let device_desc = handle.device().device_descriptor()?;
    let timeout = Duration::from_secs(1);
    let languages = handle.read_languages(timeout)?;

    println!("{:?}", device_desc);

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

fn search_device(manufacturer: &str, product: &str) -> Result<()> {
    let mut context = Context::new()?;

    for mut device in rusb::devices().unwrap().iter() {
        let device_desc = device.device_descriptor().unwrap();

        

        //let (mut device, mut handle) =

        match device.open() {
            Ok(mut handle) => {
                
                if is_correct_device(&mut handle, "SteelSeries", "SteelSeries QCK Prism Cloth") {
                    //return Ok(device_desc.vendor_id(), device_desc.product_id());
                    println!("Device found with VID {} and PID {}", device_desc.vendor_id(), device_desc.product_id());

                    /*
                    let endpoints = find_readable_endpoints(&mut device)?;
                    let endpoint = &endpoints[0];
                   //     .expect("No Configurable endpoint found on device");

                    let has_kernel_driver = match handle.kernel_driver_active(endpoint.iface) {
                        Ok(true) => {
                            handle.detach_kernel_driver(endpoint.iface)?;
                            true
                        }
                        _ => false,
                    };

                    configure_endpoint(&mut handle, &endpoint)?;
                    

                    set_report(&mut handle)?;
                    
                    let data = read_interrupt(&mut handle, endpoint.address);
                    match data {
                        Err(e) => println!("Mousepad is unhappy :( {}", e),
                        Ok(data) => println!("Mousepad is happy :) {:02X?}", data)
                    }
                    
                    set_report2(&mut handle)?;


                    handle.release_interface(endpoint.iface)?;
                    if has_kernel_driver {
                        handle.attach_kernel_driver(endpoint.iface)?;
                    }
                    */
                }
                
            },
            Err(e) => println!("{}", e)
        }

        //open_device(&mut context, device_desc.vendor_id(), device_desc.product_id()).expect("Failed to open USB device");

        //
        

        /*
        println!("Bus {:03} Device {:03} ID {:04x}:{:04x}",
            device.bus_number(),
            device.address(),
            device_desc.vendor_id(),
            device_desc.product_id());
            */

    }

    Ok(())
}

fn main() -> Result<()> {
    search_device(&"SteelSeries".to_string(), &"Steelseries QCK Prism Cloth".to_string());


    Ok(())
}