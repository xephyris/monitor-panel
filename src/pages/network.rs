use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use linux_embedded_hal::{CdevPin, I2cdev, gpio_cdev::{Chip, LineRequestFlags}};
use std::{thread, process::Command, time::{Duration, Instant}, sync::{Arc, atomic::{AtomicBool, Ordering}}};
use getifaddrs;


pub fn network_display_task() -> String {
    let mut addresses;
    let network_status = get_network_status();

    addresses = get_network_addresses();

    let mut text = format!("IP Addresses: ");
    for address in &addresses {
        text += &format!("\n{:.3}: {}", address.0, address.1);
    }
    text += &format!("\n{}", network_status.1);

    text
}

pub fn get_network_addresses() -> Vec<(String, String)> {
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

pub fn get_network_status() -> (String, String) {
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