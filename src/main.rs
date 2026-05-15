use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use linux_embedded_hal::{CdevPin, I2cdev, gpio_cdev::{Chip, LineRequestFlags}};
use embedded_hal::digital::InputPin;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306, rotation::DisplayRotation};
use sysmon_panel::pages::{self, network::{get_network_addresses, get_network_status}};
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


    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Failed to set Ctrl-C handler");

    let mut last_pressed = Instant::now();
    let mut display_on = true;

    let mut is_inverted = false;

    while running.load(Ordering::SeqCst) {

        
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

            let text = pages::network::network_display_task();

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
