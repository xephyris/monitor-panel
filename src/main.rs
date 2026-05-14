use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use linux_embedded_hal::{CdevPin, I2cdev, gpio_cdev::{Chip, LineRequestFlags}};
use embedded_hal::digital::InputPin;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306, rotation::DisplayRotation};
use std::{thread, process::Command, time::{Duration, Instant}, sync::{Arc, atomic::{AtomicBool, Ordering}}};
use getifaddrs;
use ctrlc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connected to GPIO0_CO and GPIO0_B7 (First two down from 3.3V)
    let i2c = I2cdev::new("/dev/i2c-2")?;
    let interface = I2CDisplayInterface::new(i2c);

    let mut gpio1 = Chip::new("/dev/gpiochip1")?;

    let button_star_line = gpio1.get_line(31)?;
    let handle = button_star_line.request(LineRequestFlags::INPUT, 0, "button_input").expect("Failed to configure as input");
    let mut button_star = CdevPin::new(handle)?;

    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate180)
        .into_buffered_graphics_mode();
    display.init().map_err(|e| format!("{:?}", e))?;

    display.set_brightness(Brightness::custom(1,10)).map_err(|e| format!("{:?}", e))?;

    let mut addresses;

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Failed to set Ctrl-C handler");

    let mut last_pressed = Instant::now();
    let mut display_on = true;

    let mut is_inverted = false;

    while running.load(Ordering::SeqCst) {

        let network_status = get_network_status();
        let is_pressed = button_star.is_low().expect("Read failed");
        
        if is_pressed {
            if !display_on {
                display.set_brightness(Brightness::custom(1,10)).map_err(|e| format!("{:?}", e))?;
            }
            last_pressed = Instant::now();
        }

        if last_pressed.elapsed() < Duration::from_secs(30) { 
            // println!("is_pressed {is_pressed} network {}", network_status.0);

            display.clear(BinaryColor::Off).map_err(|e| format!("{:?}", e))?;

            // Invert display periodically to prevent OLED burn in
            display.set_invert(is_inverted).map_err(|e| format!("{:?}", e))?;

            addresses = get_network_addresses();

            let mut text = format!("IP Addresses: ");
            for address in &addresses {
                text += &format!("\n{:.3}: {}", address.0, address.1);
            }
            text += &format!("\n{}", network_status.1);
            let text_style = MonoTextStyleBuilder::new()
                .font(&FONT_6X10)
                .text_color(BinaryColor::On)
                .build();

            Text::new(&text, Point::new(5, 10), text_style)
                .draw(&mut display)
                .map_err(|e| format!("{:?}", e))?;


            display.flush().map_err(|e| format!("{:?}", e))?;

            is_inverted = !is_inverted; // Swap states to prevent static burn
            
            thread::sleep(Duration::from_secs(10));
        } else {
            // Turn off display when not in use
            display.set_brightness(Brightness::custom(1,0)).map_err(|e| format!("{:?}", e))?;
            display_on = false;
        }
        thread::sleep(Duration::from_millis(200));
    }

    let _ = display.set_display_on(false);

    Ok(())
}

fn get_network_addresses() -> Vec<(String, String)> {
    let mut addresses = Vec::new();
    if let Ok(iterator) = getifaddrs::getifaddrs() {
        for interface in iterator.skip(1) {
            if interface.address.is_ipv4() && let Some(ip) = interface.address.ip_addr() {
                addresses.push((interface.name, ip.to_string()));
            }
        }
    }
    addresses.remove(0);
    addresses
}

fn get_network_status() -> (String, String) {
    let online = Command::new("ping")
        .args(["-c", "1", "-W", "2", "1.1.1.1"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    let mut ssid = "".to_string();
    if let Some(ssid_output) = Command::new("nmcli")
        .args(&["-t", "-f", "active,ssid", "dev", "wifi"])
        .output()
        .ok() {

        if ssid_output.status.success() {
            let stdout = String::from_utf8_lossy(&ssid_output.stdout);
            for line in stdout.lines() {
                if line.starts_with("yes:") {
                    ssid = line.replace("yes:", "");
                }
            }
        }
    }

    let signal = Command::new("iwconfig").arg("wlan0") // ← replace with your interface
        .output().ok()
        .and_then(|o| Some(String::from_utf8_lossy(&o.stdout).to_string()))
        .and_then(|s| 
            {
                let line = s.lines().find(|l| l.contains("Signal level=")).unwrap_or("").to_string();
                // println!("Line {:?}", &line);
                Some(line)
            })
        .and_then(|l| l.split('=').nth(2)?.split_whitespace().next()?.parse::<i32>().ok())
        .unwrap_or(-100);

    let bars = if signal >= -50 { "█████" }
        else if signal >= -60 { "████░" }
        else if signal >= -70 { "███░░" }
        else if signal >= -80 { "██░░░" }
        else if signal >= -90 { "█░░░░" }
        else { "░░░░░" };

    let status_icon = if online { "🟢 ↑" } else { "🔴 ↓" };
    let short_status_icon = if online {"^"} else {"-"};
    (format!("[{}] SSID: {:20} | {} | Signal: {} dBm", status_icon, ssid, if online { "Online" } else { "Offline" }, bars), format!("{}: {} {}", ssid, short_status_icon, signal))
}