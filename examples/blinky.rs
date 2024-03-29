#![no_std]
#![no_main]

// Warning! This examples is not working.
// Blocked by: https://github.com/georgik/esp32-buddy-rs/issues/1
//             https://github.com/esp-rs/esp-hal/issues/855
//             https://github.com/bjoernQ/esp-hal/pull/1/files
// Based on: https://github.com/espressif/esp-mdf/tree/master/examples/development_kit/buddy

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use esp_backtrace as _;
use hal::{
    clock::ClockControl, i2c, peripherals::Peripherals, prelude::*, timer::TimerGroup, Rtc, IO,
};
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1306};

use ws2812_esp32_rmt_driver::driver::color::LedPixelColorGrbw32;
use ws2812_esp32_rmt_driver::{LedPixelEsp32Rmt, RGBW8};

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let led_pin = 25;
    let mut ws2812 = LedPixelEsp32Rmt::<RGBW8, LedPixelColorGrbw32>::new(0, led_pin).unwrap();

    let sda = io.pins.gpio18;
    let scl = io.pins.gpio23;

    let i2c = i2c::I2C::new(peripherals.I2C0, sda, scl, 100u32.kHz(), &clocks);

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    Text::with_baseline("Blinky example", Point::zero(), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

    let pixels = std::iter::repeat(RGBW8::from((6, 0, 0, White(0)))).take(25);
    ws2812.write(pixels).unwrap();
    // Text::with_baseline("Hello Rust!", Point::new(0, 16), text_style, Baseline::Top)
    //     .draw(&mut display)
    //     .unwrap();

    display.flush().unwrap();

    loop {}
}
