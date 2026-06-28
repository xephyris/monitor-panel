use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use linux_embedded_hal::{CdevPin, I2cdev, gpio_cdev::{Chip, LineRequestFlags}};
use embedded_hal::digital::InputPin;
use ssd1306::{I2CDisplayInterface, Ssd1306, mode::BufferedGraphicsMode, prelude::*, rotation::DisplayRotation};
use sysmon_panel::{draw_text_with_icons, pages::{self, Page, monitor::UsagePage, network::NetworkPage}};
use std::{thread, process::Command, time::{Duration, Instant}, sync::{Arc, atomic::{AtomicBool, Ordering}}};
use getifaddrs;
use ctrlc;

use ab_glyph::{point, Font, FontRef, Glyph};

fn main() -> Result<(), Box<dyn std::error::Error>> {


    // Connected to GPIO0_CO and GPIO0_B7 (First two down from 3.3V)
    let i2c = I2cdev::new("/dev/i2c-2")?;
    let interface = I2CDisplayInterface::new(i2c);

    let mut gpio1 = Chip::new("/dev/gpiochip1")?;
    let mut gpio3 = Chip::new("/dev/gpiochip3")?;

    let button_star_line = gpio1.get_line(31)?;
    let handle_star = button_star_line.request(LineRequestFlags::INPUT, 0, "button_input").expect("Failed to configure as input");
    let mut button_star = CdevPin::new(handle_star)?;

    let button_hash_line = gpio3.get_line(3)?;
    let handle_hash = button_hash_line.request(LineRequestFlags::INPUT, 0, "button_input").expect("Failed to configure as input");
    let mut button_hash = CdevPin::new(handle_hash)?;

    let button_up_line = gpio3.get_line(18)?;
    let handle_up = button_up_line.request(LineRequestFlags::INPUT, 0, "button_input").expect("Failed to configure as input");
    let mut button_up = CdevPin::new(handle_up)?;

    let button_down_line = gpio3.get_line(2)?;
    let handle_down = button_down_line.request(LineRequestFlags::INPUT, 0, "button_input").expect("Failed to configure as input");
    let mut button_down = CdevPin::new(handle_down)?;

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
    let mut last_invert = Instant::now();
    
    let mut pages: Vec<Box<dyn Page>>= vec![Box::new(NetworkPage::new(vec![true])), Box::new(UsagePage::new())];
    let mut current_page: usize = 0;
    let mut main_menus = true;

    let font = FontRef::try_from_slice(include_bytes!("../lucide-font/lucide.ttf")).unwrap();

    while running.load(Ordering::SeqCst) {

        
        let star_pressed = button_star.is_low().expect("Read failed");
        let hash_pressed = button_hash.is_low().expect("Read failed");
        let up_pressed = button_up.is_low().expect("Read failed");
        let down_pressed = button_down.is_low().expect("Read failed");
        
        if star_pressed || hash_pressed || up_pressed || down_pressed{
            if !display_on {
                display.set_brightness(Brightness::custom(1,10)).map_err(|e| format!("{:?}", e))?;
                display_on = true;
            } else if main_menus{
                if up_pressed {
                    println!("current_page new {} {}", ((current_page as i32) - 1), pages.len());
                    current_page = (((current_page as i32) - 1).rem_euclid(pages.len() as i32)) as usize;
                    println!("UP Button Pressed");
                } else if down_pressed {
                    current_page = (((current_page as i32) - 1).rem_euclid(pages.len() as i32)) as usize;
                    println!("DOWN Button Pressed");
                }
            }
            last_pressed = Instant::now();
        }

        if last_pressed.elapsed() < Duration::from_secs(30) { 
            // println!("is_pressed {is_pressed} network {}", network_status.0);
            pages[current_page].update();
            display.clear(BinaryColor::Off).map_err(|e| format!("{:?}", e))?;

            if last_invert.elapsed() > Duration::from_secs(4) {
                // Invert display periodically to prevent OLED burn in
                display.set_invert(is_inverted).map_err(|e| format!("{:?}", e))?;
                is_inverted = !is_inverted; 
                last_invert = Instant::now();
            }

            let text = pages[current_page].display();

            draw_text_with_icons(&mut display, &font, &text, 5, 10)?;

            display.flush().map_err(|e| format!("{:?}", e))?;

            
            thread::sleep(pages[current_page].refresh_rate());
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
