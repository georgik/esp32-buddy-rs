#![no_std]
#![no_main]

// Warning! This examples is not working. Blocked by - https://github.com/georgik/esp32-buddy-rs/issues/1
// Based on: https://github.com/espressif/esp-mdf/tree/master/examples/development_kit/buddy

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use esp_println::println;
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1306};
use hal::{clock::ClockControl, i2c, IO, peripherals::Peripherals, prelude::*, timer::TimerGroup, Rtc, Delay, rmt::Rmt};
use esp_backtrace as _;

use smart_leds::{
    brightness,
    gamma,
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite,
};

use hal::gpio::wrappers::InvertedOutputPin;

use esp_hal_smartled::{smartLedAdapter, SmartLedsAdapter};

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let mut system = peripherals.DPORT.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(
        peripherals.TIMG1,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt1 = timer_group1.wdt;


    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let led_pin = 25;
    // let mut ws2812 = LedPixelEsp32Rmt::<RGBW8, LedPixelColorGrbw32>::new(0, led_pin).unwrap();
    let rmt = Rmt::new(
        peripherals.RMT,
        80u32.MHz(),
        &mut system.peripheral_clock_control,
        &clocks,
    ).unwrap();

    let sda = io.pins.gpio18;
    let scl = io.pins.gpio23;

    let i2c = i2c::I2C::new(
        peripherals.I2C0,
        sda,
        scl,
        100u32.kHz(),
        &mut system.peripheral_clock_control,
        &clocks,
    );
    let mut delay = Delay::new(&clocks);
    // let interface = I2CDisplayInterface::new(i2c);
    // let mut display = Ssd1306::new(
    //     interface,
    //     DisplaySize128x32,
    //     DisplayRotation::Rotate0,
    // ).into_buffered_graphics_mode();
    // display.init().unwrap();

    // let text_style = MonoTextStyleBuilder::new()
    //     .font(&FONT_6X10)
    //     .text_color(BinaryColor::On)
    //     .build();

    // Text::with_baseline("Blinky example", Point::zero(), text_style, Baseline::Top)
    //     .draw(&mut display)
    //     .unwrap();

        let mut color = Hsv {
            hue: 0,
            sat: 255,
            val: 255,
        };
        let mut data;
        let mut led = <smartLedAdapter!(0, 1)>::new(rmt.channel0,  InvertedOutputPin::new(io.pins.gpio25));
        loop {
            // Iterate over the rainbow!
            for hue in 0..=255 {
                color.hue = hue;
                // Convert from the HSV color space (where we can easily transition from one
                // color to the other) to the RGB color space that we can then send to the LED
                let rgb_color = hsv2rgb(color);
                println!("RGB: {:?}", rgb_color);
    
                // Assign new color to all 12 LEDs
                data = [rgb_color; 1];
    
                // When sending to the LED, we do a gamma correction first (see smart_leds
                // documentation for details) and then limit the brightness to 10 out of 255 so
                // that the output it's not too bright.
                led.write(brightness(gamma(data.iter().cloned()), 10))
                    .unwrap();
                delay.delay_ms(20u8);
            }
        }
    // let pixels = std::iter::repeat(RGBW8::from((6, 0, 0, White(0)))).take(25);
    // ws2812.write(pixels).unwrap();
    // Text::with_baseline("Hello Rust!", Point::new(0, 16), text_style, Baseline::Top)
    //     .draw(&mut display)
    //     .unwrap();

    // display.flush().unwrap();

    // loop {}
}
