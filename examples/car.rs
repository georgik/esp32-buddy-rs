// ESP-Buddy HW: // Based on: https://github.com/espressif/esp-mdf/tree/master/examples/development_kit/buddy

#![no_std]
#![no_main]

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, ascii::FONT_4X6, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text, Alignment},
};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

use hal::{
    clock::ClockControl,
    peripherals::Peripherals,
    prelude::*,
    Delay,
    IO,
    i2c
};

use esp_backtrace as _;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Initialize the Delay peripheral, and use it to toggle the LED state in a
    // loop.
    let mut delay = Delay::new(&clocks);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let sda = io.pins.gpio18;
    let scl = io.pins.gpio23;

    let i2c = i2c::I2C::new(
        peripherals.I2C0,
        sda,
        scl,
        100u32.kHz(),
        &clocks,
    );

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(
        interface,
        DisplaySize128x32,
        DisplayRotation::Rotate0,
    ).into_buffered_graphics_mode();
    display.init().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    let espressif_style = MonoTextStyleBuilder::new()
        .font(&FONT_4X6)
        .text_color(BinaryColor::On)
        .build();

    loop {
        // Iterate over the rainbow!
        for position_x in -53..=210 {
            display.clear();

            //back + spoiler

            Text::with_baseline("_", Point::new(position_x - 12, 1), text_style, Baseline::Top)
                .draw(&mut display)
                .unwrap();

            Text::with_baseline("_", Point::new(position_x - 10, 1), text_style, Baseline::Top)
                .draw(&mut display)
                .unwrap();

            Text::with_baseline("\\", Point::new(position_x - 9, 8), text_style, Baseline::Top)
                .draw(&mut display)
                .unwrap();

            Text::with_baseline("P", Point::new(position_x - 7, 14), text_style, Baseline::Top)
                .draw(&mut display)
                .unwrap();

            Text::with_baseline("|", Point::new(position_x - 9, 21), text_style, Baseline::Top)
                .draw(&mut display)
                .unwrap();

            //chassis + label

            Text::with_baseline("-", Point::new(position_x - 6, 24), text_style, Baseline::Top)
                .draw(&mut display)
                .unwrap();

            Text::with_baseline("O", Point::new(position_x, 24), text_style, Baseline::Top)
                .draw(&mut display)
                .unwrap();

            Text::with_baseline("Espressif", Point::new(position_x, 18), espressif_style, Baseline::Top)
                .draw(&mut display)
                .unwrap();

            for i in (6..30).step_by(4) {
                Text::with_baseline("_", Point::new(position_x + i, 20), text_style, Baseline::Top)
                    .draw(&mut display)
                    .unwrap();
            }

            Text::with_baseline("O", Point::new(position_x + 30, 24), text_style, Baseline::Top)
                .draw(&mut display)
                .unwrap();

            for i in (36..44).step_by(4) {
                Text::with_baseline("_", Point::new(position_x + i, 20), text_style, Baseline::Top)
                .draw(&mut display)
                .unwrap();
            }

            //front

            Text::with_baseline("|", Point::new(position_x + 43, 21), text_style, Baseline::Top)
                .draw(&mut display)
                .unwrap();

            Text::with_baseline("\\", Point::new(position_x + 41, 14), text_style, Baseline::Top)
                .draw(&mut display)
                .unwrap();

            for i in (-3..37).step_by(2) {
                Text::with_baseline("_", Point::new(position_x + i, 7), text_style, Baseline::Top)
                    .draw(&mut display)
                    .unwrap();
                Text::with_baseline("_", Point::new(position_x + i, 8), text_style, Baseline::Top)
                    .draw(&mut display)
                    .unwrap();
            }

            //cabin

            Text::with_baseline("\\", Point::new(position_x + 26, 8), text_style, Baseline::Top)
                .draw(&mut display)
                .unwrap();

            //driver

            Text::with_baseline("(", Point::new(position_x + 13, 4), text_style, Baseline::Top)
                .draw(&mut display)
                .unwrap();
            Text::with_baseline(")", Point::new(position_x + 17, 4), text_style, Baseline::Top)
                .draw(&mut display)
                .unwrap();
            Text::with_baseline("|", Point::new(position_x + 15, 10), text_style, Baseline::Top)
                .draw(&mut display)
                .unwrap();

            // trailer

            for i in (-22..-9).step_by(4) {
                Text::with_baseline("_", Point::new(position_x + i, 15), text_style, Baseline::Top)
                    .draw(&mut display)
                    .unwrap();
            }

            for i in (0..24).step_by(4) {
                Text::with_baseline("|", Point::new(position_x - 77, i), text_style, Baseline::Top)
                    .draw(&mut display)
                    .unwrap();
            }

            for i in (-75..-25).step_by(4) {
                Text::with_baseline("-", Point::new(position_x + i, -3), text_style, Baseline::Top)
                    .draw(&mut display)
                    .unwrap();
            }

            Text::with_alignment("car\nanimation\nexample", Point::new(position_x - 50, 8), espressif_style, Alignment::Center)
                .draw(&mut display)
                .unwrap();

            for i in (0..24).step_by(4) {
                Text::with_baseline("|", Point::new(position_x - 24, i), text_style, Baseline::Top)
                    .draw(&mut display)
                    .unwrap();
            }

            //trailer chassis

            for i in (-75..-70).step_by(2) {
                Text::with_baseline("_", Point::new(position_x + i, 20), text_style, Baseline::Top)
                    .draw(&mut display)
                    .unwrap();
            }

            Text::with_baseline("O", Point::new(position_x + -65, 24), text_style, Baseline::Top)
                .draw(&mut display)
                .unwrap();

            for i in (-60..-35).step_by(5) {
                Text::with_baseline("_", Point::new(position_x + i, 20), text_style, Baseline::Top)
                    .draw(&mut display)
                    .unwrap();
            }

            Text::with_baseline("O", Point::new(position_x + -35, 24), text_style, Baseline::Top)
                .draw(&mut display)
                .unwrap();

            for i in (-30..-25).step_by(2) {
                Text::with_baseline("_", Point::new(position_x + i, 20), text_style, Baseline::Top)
                    .draw(&mut display)
                    .unwrap();
            }


            display.flush().unwrap();
            delay.delay_ms(25u32);
        }
    }
}
