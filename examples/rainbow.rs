// Warning! This examples is not working. Blocked by - https://github.com/georgik/esp32-buddy-rs/issues/1
// Based on https://github.com/esp-rs/esp-hal/blob/main/esp32-hal/examples/hello_rgb.rs
// ESP-Buddy HW: // Based on: https://github.com/espressif/esp-mdf/tree/master/examples/development_kit/buddy

//! RGB LED Demo
//!
//! This example drives an 12-element RGB ring that is connected to GPIO33
//!
//! The LEDs in the ring are transitioning though the HSV color spectrum for
//! - Saturation: 255
//! - Hue: 0 - 255
//! - Value: 255
//!
//! For the 12-element RGB ring to work, building the release version is going
//! to be required.

#![no_std]
#![no_main]


use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

use esp32_hal::{
    clock::ClockControl,
    pac,
    prelude::*,
    timer::TimerGroup,
    utils::{smartLedAdapter, SmartLedsAdapter},
    Delay,
    PulseControl,
    Rtc,
    IO,
    i2c
};
#[allow(unused_imports)]
use esp_backtrace as _;
use smart_leds::{
    brightness,
    gamma,
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite,
};
use xtensa_lx_rt::entry;
use heapless::String;

#[entry]
fn main() -> ! {
    let peripherals = pac::Peripherals::take().unwrap();
    let mut system = peripherals.DPORT.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt = timer_group0.wdt;
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    // Disable MWDT and RWDT (Watchdog) flash boot protection
    wdt.disable();
    rtc.rwdt.disable();

    // Configure RMT peripheral globally
    let pulse = PulseControl::new(peripherals.RMT, &mut system.peripheral_clock_control).unwrap();

    // We use one of the RMT channels to instantiate a `SmartLedsAdapter` which can
    // be used directly with all `smart_led` implementations
    // -> We need to use the macro `smartLedAdapter!` with the number of addressed
    // LEDs here to initialize the internal LED pulse buffer to the correct
    // size!
    let mut led = <smartLedAdapter!(12)>::new(pulse.channel1, io.pins.gpio25);

    // Initialize the Delay peripheral, and use it to toggle the LED state in a
    // loop.
    let mut delay = Delay::new(&clocks);

    let sda = io.pins.gpio18;
    let scl = io.pins.gpio23;

    let i2c = i2c::I2C::new(
        peripherals.I2C0,
        sda,
        scl,
        100u32.kHz(),
        &mut system.peripheral_clock_control,
        &clocks,
    )
    .unwrap();

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(
        interface,
        DisplaySize128x32,
        DisplayRotation::Rotate0,
    ).into_buffered_graphics_mode();
    display.init().unwrap();

    let mut color = Hsv {
        hue: 0,
        sat: 255,
        val: 255,
    };
    let mut data;

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    loop {
        // Iterate over the rainbow!
        for hue in 0..=255 {
            display.clear();
            Text::with_baseline("Rainbow example", Point::zero(), text_style, Baseline::Top)
              .draw(&mut display)
              .unwrap();

            color.hue = hue;
            // Convert from the HSV color space (where we can easily transition from one
            // color to the other) to the RGB color space that we can then send to the LED
            let rgb_color = hsv2rgb(color);

            // Assign new color to all 12 LEDs
            data = [rgb_color; 12];

            // When sending to the LED, we do a gamma correction first (see smart_leds
            // documentation for details) and then limit the brightness to 10 out of 255 so
            // that the output it's not too bright.
            led.write(brightness(gamma(data.iter().cloned()), 10))
                .unwrap();

            let hue_string:String<32> = String::from(hue);
            Text::with_baseline(&hue_string, Point::new(0, 16), text_style, Baseline::Top)
                .draw(&mut display)
                .unwrap();

            display.flush().unwrap();
            delay.delay_ms(50u32);
        }
    }
}
