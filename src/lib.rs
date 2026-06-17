pub mod pages;

// Icon Font from Lucide Icons (MIT)

use ab_glyph::{Font, FontRef, Glyph, ScaleFont, point};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use ssd1306::{I2CDisplayInterface, Ssd1306, mode::BufferedGraphicsMode, prelude::*, rotation::DisplayRotation};


pub fn draw_text_with_icons<DI, SIZE>(
    display: &mut Ssd1306<DI, SIZE, BufferedGraphicsMode<SIZE>>,
    icon_font: &FontRef,
    text: &str,
    start_x: i32,
    start_y: i32,
) -> Result<(), String>
where
    DI: WriteOnlyDataCommand,
    SIZE: DisplaySize,
{
    let mut current_x = start_x;
    let mut current_y = start_y;

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    let char_width = 6;
    let line_height = 12; 

    let icon_scale = 11.0;

    for character in text.chars() {
        if character == '\n' {
            current_x = start_x;
            current_y += line_height;
            continue;
        }

        if matches!(character, '\u{E000}'..='\u{F8FF}') {
            let glyph_id = icon_font.glyph_id(character);
            
            let glyph: Glyph = glyph_id.with_scale_and_position(
                icon_scale, 
                point(current_x as f32, (current_y + 8) as f32)
            );

            if let Some(outlined_glyph) = icon_font.outline_glyph(glyph) {
                let bounds = outlined_glyph.bounds();
                outlined_glyph.draw(|pixel_x, pixel_y, coverage| {
                    if coverage > 0.5 {
                        let screen_x = (bounds.min.x + pixel_x as f32) as i32;
                        let screen_y = (bounds.min.y + pixel_y as f32 - icon_scale/2.0) as i32;

                        if screen_x >= 0 && screen_x < 128 && screen_y >= 0 && screen_y < 64 {
                            Pixel(Point::new(screen_x, screen_y), BinaryColor::On)
                                .draw(display)
                                .unwrap();
                        }
                    }
                });
            }
            
            current_x += icon_font.as_scaled(icon_scale).h_advance(glyph_id) as i32;

        } else {
            let single_char_str = character.to_string();
            
            Text::new(&single_char_str, Point::new(current_x, current_y), text_style)
                .draw(display)
                .map_err(|e| format!("{:?}", e))?;

            current_x += char_width;
        }
    }

    Ok(())
}