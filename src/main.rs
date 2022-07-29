use rusb::UsbContext;
use simple_log::LogConfigBuilder;
use std::process::Command;

#[derive(Debug, PartialEq, Eq)]
enum Signal {
    ChangeAzerty,
    ChangeQwerty,
    NothingChanged,
}

fn main() {
    let mut is_connected = false;
    let mut last_signal = Signal::NothingChanged;
    let config = LogConfigBuilder::builder()
        .size(100)
        .roll_count(10)
        .level("debug")
        .output_console()
        .build();

    if let Err(..) = simple_log::new(config) {
        println!("Failed to initialize logger");
    }

    loop {
        let res = handle_usb_switch_logic(&mut is_connected);
        let mut signal = Signal::NothingChanged;

        match res {
            Err(str) => {
                simple_log::error!("{}", str);
                continue;
            }
            Ok(sig) => {
                signal = sig;
            }
        }

        if signal != last_signal {
            let res_change = change_keyboard_layout(&signal);

            if let Err(str) = res_change {
                simple_log::error!("{}", str);
                continue;
            }

            last_signal = signal;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

/// Change the keyboard layout according to the signal.
/// Returns an error if the keyboard layout could not be changed.
/// Returns Ok(()) if the keyboard layout was changed.
/// Arguments:
/// - signal: the signal to change the keyboard layout.
/// Returns:
/// - an error if the keyboard layout could not be changed.
/// - Ok(()) if the keyboard layout was changed.
fn change_keyboard_layout(signal: &Signal) -> std::result::Result<(), &'static str> {
    match signal {
        Signal::ChangeAzerty => {
            let command = "/usr/bin/setxkbmap fr";
            let output = Command::new("/bin/sh").arg("-c").arg(command).output();

            match output {
                Err(str) => {
                    simple_log::error!("{}", str);
                    return Err("Failed to change keyboard layout");
                }
                Ok(out) => {
                    if out.status.success() {
                        simple_log::info!("Successfully changed keyboard layout to azerty");
                    } else {
                        simple_log::error!("{}", String::from_utf8_lossy(&out.stderr));
                        return Err("Failed to change keyboard layout to azerty");
                    }
                }
            }
        }

        Signal::ChangeQwerty => {
            let command = "/usr/bin/setxkbmap us";
            let output = Command::new("/bin/sh").arg("-c").arg(command).output();

            match output {
                Err(str) => {
                    simple_log::error!("{}", str);
                    return Err("Failed to change keyboard layout");
                }
                Ok(out) => {
                    if out.status.success() {
                        simple_log::info!("Successfully changed keyboard layout to azerty");
                    } else {
                        simple_log::error!("{}", String::from_utf8_lossy(&out.stderr));
                        return Err("Failed to change keyboard layout to azerty");
                    }
                }
            }
        }

        Signal::NothingChanged => {}
    }
    Ok(())
}

/// Handles the logic of the USB switch.
/// Returns an error if something went wrong.
/// Returns Ok(()) if everything went fine.
/// Arguments
/// - `is_connected` - A mutable boolean that is set to true if the USB switch is connected.
/// Returns:
/// - an error if something went wrong.
/// - Ok(()) if everything went fine.
fn handle_usb_switch_logic(is_connected: &mut bool) -> std::result::Result<Signal, &'static str> {
    let devices = get_usb_devices().map_err(|_| "Failed to get USBdevices")?;
    let target_id = "445a:1121";
    let mut found = false;

    devices.iter().for_each(|device| {
        let desc = device.device_descriptor();
        match desc {
            Ok(desc) => {
                let id = format!("{:04x}:{:04x}", desc.vendor_id(), desc.product_id());
                if id == target_id {
                    found = true;
                }
            }
            Err(str) => {
                simple_log::error!("{}", str);
            }
        }
    });

    if found && !(*is_connected) {
        *is_connected = true;

        return Ok(Signal::ChangeQwerty);
    }

    if !found && *is_connected {
        *is_connected = false;

        return Ok(Signal::ChangeAzerty);
    }

    Ok(Signal::NothingChanged)
}

/// Gets all USB devices.
/// Returns an error if something went wrong.
/// Returns Ok(Vec<rusb::Device>) if everything went fine.
/// Arguments:
/// - None.
/// Returns:
/// - an error if something went wrong.
/// - Ok(Vec<rusb::Device>) if everything went fine.
fn get_usb_devices() -> std::result::Result<rusb::DeviceList<rusb::Context>, &'static str> {
    let devices = rusb::Context::new().map_err(|_| "Failed to create USB context")?;
    let devices = devices.devices().map_err(|_| "Failed to get USB devices")?;

    Ok(devices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_keyboard_layout() {
        let res = change_keyboard_layout(&Signal::ChangeAzerty);
        assert!(res.is_ok());
        let res = change_keyboard_layout(&Signal::ChangeQwerty);
        assert!(res.is_ok());
        let res = change_keyboard_layout(&Signal::NothingChanged);
        assert!(res.is_ok());
    }

    #[test]
    fn test_handle_usb_switch_logic() {
        let mut is_connected = false;
        let res = handle_usb_switch_logic(&mut is_connected);
        assert!(res.is_ok());
    }

    #[test]
    fn test_get_usb_devices() {
        let res = get_usb_devices();
        assert!(res.is_ok());
    }
}
