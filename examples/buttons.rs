#![no_std]
#![no_main]

// Based on: https://github.com/espressif/esp-mdf/tree/master/examples/development_kit/buddy

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use esp_backtrace as _;
use hal::{
    delay::Delay,
    gpio::{Input, Io, Level, Output, Pin, Pull},
    i2c,
    prelude::*,
};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

#[entry]
fn main() -> ! {
    let peripherals = hal::init(hal::Config::default());

    let mut delay = Delay::new();

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let sda = io.pins.gpio18;
    let scl = io.pins.gpio23;

    let i2c = i2c::I2c::new(peripherals.I2C0, sda, scl, 100u32.kHz());

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    let button_left_pin = io.pins.gpio0.degrade();
    let button_left_pin = Input::new(button_left_pin, Pull::Up);
    let button_right_pin = io.pins.gpio4.degrade();
    let button_right_pin = Input::new(button_right_pin, Pull::Up);

    // let button_right_pin = io.pins.gpio4.into_pull_down_input();

    loop {
        display.clear();
        Text::with_baseline("Buttons example", Point::zero(), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();

        if button_left_pin.is_low() {
            Text::with_baseline("Left", Point::new(0, 16), text_style, Baseline::Top)
                .draw(&mut display)
                .unwrap();
        }

        if button_right_pin.is_low() {
            Text::with_baseline("Right", Point::new(60, 16), text_style, Baseline::Top)
                .draw(&mut display)
                .unwrap();
        }

        display.flush().unwrap();
        delay.delay_millis(30u32);
        // delay.delay_millis(25u32);
    }
}
