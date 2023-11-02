// ESP-Buddy HW: // Based on: https://github.com/espressif/esp-mdf/tree/master/examples/development_kit/buddy

#![no_std]
#![no_main]


use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
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
#[allow(unused_imports)]
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

    loop {
        // Iterate over the rainbow!
        for position_x in -30..=128 {
            display.clear();
            Text::with_baseline("Animation example", Point::zero(), text_style, Baseline::Top)
              .draw(&mut display)
              .unwrap();


            Text::with_baseline("_-=]>", Point::new(position_x, 16), text_style, Baseline::Top)
                .draw(&mut display)
                .unwrap();
            Text::with_baseline("<[=-_", Point::new(128-position_x-30, 16), text_style, Baseline::Top)
                .draw(&mut display)
                .unwrap();

            display.flush().unwrap();
            delay.delay_ms(25u32);
        }
    }
}
