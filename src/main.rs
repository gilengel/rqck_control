use clap::{arg_enum, clap_app, value_t, values_t};
use rusb::{Context, Device, DeviceDescriptor, DeviceHandle, Result, UsbContext};

#[derive(Debug)]
struct Endpoint {
    config: u8,
    iface: u8,
    setting: u8,
    address: u8,
}



mod commands;
use commands::{ColorRGB, Zone, apply_changes, disable_lower_zone, disable_upper_zone, set_intensity, switch_mode_to_steady};

fn main() -> Result<()> {
    match Context::new() {
        Ok(mut context) => match open_device(&mut context, 4152, 5389) {
            Some((mut device, _, mut handle)) => {
                let endpoints = find_readable_endpoints(&mut device)?;
                let endpoint = endpoints
                    .first()
                    .expect("No Configurable endpoint found on device");
                let has_kernel_driver = match handle.kernel_driver_active(endpoint.iface) {
                    Ok(true) => {
                        handle.detach_kernel_driver(endpoint.iface)?;
                        true
                    }
                    _ => false,
                };

                configure_endpoint(&mut handle, endpoint)?;

                let matches = clap_app!(myapp =>
                    (about: "Controls your Steelseries QCK Cloth mousepad")
                    (@subcommand set_intensity =>
                        (about: "Sets the intensity of the LEDs")
                        (@arg INTENSITY: +required +takes_value -i --intensity "value between 0 and 100 for the intensity")
                    )

                    (@subcommand disable =>
                        (about: "Disables one of the two zones of the LEDs on your pad")
                        (@arg ZONE: +required ... "Zones you want to disable")
                    )

                    (@subcommand solid =>
                        (about: "Disables one of the two zones of the LEDs on your pad")
                        (@arg ZONE: +required  "Zones you want to disable")
                        (@arg RED: +required  "Amount of red color")
                        (@arg GREEN: +required  "Amount of green color")
                        (@arg BLUE: +required "Amount of blue color")
                    )
                ).get_matches();

                if let Some(matches) = matches.subcommand_matches("set_intensity") {
                    let intensity = matches.value_of("INTENSITY").unwrap();

                    match intensity.parse::<i16>() {
                        Ok(intensity) => {
                            let range = 0..101;

                            if !range.contains(&intensity) {
                                println!("The provided value for INTENSITY is not in the range from 0 to 100.");
                            }

                            let intensity = (intensity as f32 * 2.55) as u8;
                            set_intensity(&mut handle, intensity)?;
                        }
                        Err(_) => println!("The provided value for INTENSITY is not a number"),
                    }
                }

                if let Some(matches) = matches.subcommand_matches("disable") {
                    let zones = values_t!(matches, "ZONE", Zone).unwrap_or_else(|e| e.exit());

                    for zone in zones {
                        match zone {
                            Zone::Upper => {
                                disable_upper_zone(&mut handle)?;
                                apply_changes(&mut handle)?;
                            }
                            Zone::Lower => {
                                disable_lower_zone(&mut handle)?;
                                apply_changes(&mut handle)?;
                            }
                        }
                    }
                }

                if let Some(matches) = matches.subcommand_matches("solid") {
                    let zone = value_t!(matches, "ZONE", Zone).unwrap_or_else(|e| e.exit());
                    let red = matches.value_of("RED").unwrap();
                    let green = matches.value_of("GREEN").unwrap();
                    let blue = matches.value_of("BLUE").unwrap();
                    
                    if let (Ok(red), Ok(green), Ok(blue)) = (red.parse::<u8>(), green.parse::<u8>(), blue.parse::<u8>()) {
                        let range = 0..=255;

                        if !range.contains(&red) || !range.contains(&green) || !range.contains(&blue)  {
                            println!("The provided values for RED, GREEN, BLUE must be positive numbers are not in the range of 0-255");
                        } 

                        switch_mode_to_steady(&mut handle, zone, ColorRGB::new(red, green, blue))?;
                        apply_changes(&mut handle)?;                   
                    }
                }


                // cleanup after use
                handle.release_interface(endpoint.iface)?;
                if has_kernel_driver {
                    handle.attach_kernel_driver(endpoint.iface)?;
                }
            }
            None => println!("could not find device {:04x}:{:04x}", 4152, 5389),
        },
        Err(e) => panic!("could not initialize libusb: {}", e),
    }

    Ok(())
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

// returns all readable endpoints for given usb device and descriptor
fn find_readable_endpoints<T: UsbContext>(device: &mut Device<T>) -> Result<Vec<Endpoint>> {
    let device_desc = device.device_descriptor()?;
    let mut endpoints = vec![];
    for n in 0..device_desc.num_configurations() {
        let config_desc = match device.config_descriptor(n) {
            Ok(c) => c,
            Err(_) => continue,
        };
        // println!("{:#?}", config_desc);
        for interface in config_desc.interfaces() {
            for interface_desc in interface.descriptors() {
                // println!("{:#?}", interface_desc);
                for endpoint_desc in interface_desc.endpoint_descriptors() {
                    // println!("{:#?}", endpoint_desc);
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
    //handle.set_active_configuration(endpoint.config)?;
    handle.claim_interface(endpoint.iface)?;
    handle.set_alternate_setting(endpoint.iface, endpoint.setting)
}
