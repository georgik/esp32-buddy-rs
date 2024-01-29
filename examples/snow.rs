#![no_std]
#![no_main]

use embedded_graphics::{
    mono_font::{ascii::FONT_4X6, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use hal::{clock::ClockControl, i2c, peripherals::Peripherals, prelude::*, Delay, Rng, IO};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

use esp_backtrace as _;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let mut delay = Delay::new(&clocks);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let sda = io.pins.gpio18;
    let scl = io.pins.gpio23;

    let i2c = i2c::I2C::new(peripherals.I2C0, sda, scl, 100u32.kHz(), &clocks);

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate180)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    let snow_style = MonoTextStyleBuilder::new()
        .font(&FONT_4X6)
        .text_color(BinaryColor::On)
        .build();

    // Instantiate the hardware RNG:
    let mut rng = Rng::new(peripherals.RNG);

    // Number of snowflakes and their positions
    let mut snowflakes = [(0, 0); 10]; // Adjust the number of snowflakes here

    loop {
        display.clear();

        // Update snowflake positions
        for snowflake in snowflakes.iter_mut() {
            // Randomly generate new snowflakes at the top
            if rng.random() % 20 == 0 {
                snowflake.0 = (rng.random() % 128) as i32;
                snowflake.1 = 0;
            } else {
                // Adjust for 45-degree tilt
                snowflake.1 += 1;
                snowflake.0 -= 1; // Adjust this value if needed for correct tilt compensation

                // Check bounds and reset if needed
                if snowflake.1 > 31 {
                    snowflake.1 = 0;
                    snowflake.0 = (rng.random() % 128) as i32; // Reset x position too
                }
                if snowflake.0 < 0 {
                    snowflake.0 = 127; // Wrap around if it goes off the left edge
                }
            }

            // Draw snowflake
            Text::with_baseline(
                "*",
                Point::new(snowflake.0, snowflake.1),
                snow_style,
                Baseline::Top,
            )
            .draw(&mut display)
            .unwrap();
        }

        display.flush().unwrap();
        delay.delay_ms(100u32); // Adjust for snowflake fall speed
    }
}
