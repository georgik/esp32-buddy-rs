#![no_std]
#![no_main]

// Based on: https://github.com/espressif/esp-mdf/tree/master/examples/development_kit/buddy

use core::fmt::Write;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use esp_backtrace as _;
use esp_println::println;
use hal::{clock::ClockControl, i2c, peripherals::Peripherals, prelude::*, Delay, IO};
use shared_bus::BusManagerSimple;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

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

    // We need to access two peripherals on I2C
    // based on example: https://github.com/ferrous-systems/espressif-trainings/blob/main/advanced/i2c-sensor-reading/examples/part_2.rs
    let bus = shared_bus::BusManagerSimple::new(i2c);

    let proxy_1 = bus.acquire_i2c();
    let mut proxy_2 = bus.acquire_i2c();

    let interface = I2CDisplayInterface::new(proxy_1);
    let mut display = Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    let mut hts221 = hts221::Builder::new().build(&mut proxy_2).unwrap();

    display.flush().unwrap();

    loop {
        display.clear();
        Text::with_baseline(
            "Temperature/Humidity",
            Point::zero(),
            text_style,
            Baseline::Top,
        )
        .draw(&mut display)
        .unwrap();

        // Acquire measurement and perform correction - https://crates.io/crates/hts221
        let rh = hts221.humidity_x2(&mut proxy_2).unwrap() / 2;
        let deg_c = hts221.temperature_x8(&mut proxy_2).unwrap() / 8;

        let mut rh_string: heapless::String<32> = heapless::String::new();
        let mut deg_string: heapless::String<32> = heapless::String::new();

        write!(rh_string, "{}%", rh).unwrap();
        write!(deg_string, "{} C", deg_c).unwrap();

        println!("{} {}", rh_string, deg_string);

        Text::with_baseline(&deg_string, Point::new(0, 16), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();
        Text::with_baseline(&rh_string, Point::new(72, 16), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();

        display.flush().unwrap();
        delay.delay_ms(5000u32);
    }
}
