#![no_std]
#![no_main]

// Based on: https://github.com/espressif/esp-mdf/tree/master/examples/development_kit/buddy

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use esp32_hal::{clock::ClockControl, Delay, i2c, IO, pac::Peripherals, prelude::*, timer::TimerGroup, Rtc};
use esp_backtrace as _;
use xtensa_lx_rt::entry;

fn gpio_state<D>(target: &mut D, gpio_number:i32, state:bool)
where
    D: DrawTarget<Color = BinaryColor>
{
    let pos_y = (gpio_number / 16) * 10 + 10;
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();
    let text_state = { if state {"1"} else {"0"}};
    Text::with_baseline(text_state, Point::new(8*(gpio_number % 16), pos_y ), text_style, Baseline::Top).draw(target);
}

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let mut system = peripherals.DPORT.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    let mut wdt1 = timer_group1.wdt;


    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    let mut delay = Delay::new(&clocks);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

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

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();


    let button_0_pin = io.pins.gpio0.into_pull_up_input();
    let button_1_pin = io.pins.gpio1.into_pull_up_input();
    let button_2_pin = io.pins.gpio2.into_pull_up_input();
    let button_3_pin = io.pins.gpio3.into_pull_up_input();
    let button_4_pin = io.pins.gpio4.into_pull_up_input();
    let button_5_pin = io.pins.gpio5.into_pull_up_input();
    // let button_6_pin = io.pins.gpio6.into_pull_up_input(); // Used
    // let button_7_pin = io.pins.gpio7.into_pull_up_input(); // Used
    // let button_8_pin = io.pins.gpio8.into_pull_up_input(); // Used
    let button_9_pin = io.pins.gpio9.into_pull_up_input();

    // let button_10_pin = io.pins.gpio10.into_pull_up_input(); // Used
    // let button_11_pin = io.pins.gpio11.into_pull_up_input(); // Used
    let button_12_pin = io.pins.gpio12.into_pull_up_input();
    let button_13_pin = io.pins.gpio13.into_pull_up_input();
    let button_14_pin = io.pins.gpio14.into_pull_up_input();
    let button_15_pin = io.pins.gpio15.into_pull_up_input();
    let button_16_pin = io.pins.gpio16.into_pull_up_input();
    let button_17_pin = io.pins.gpio17.into_pull_up_input();
    // let button_18_pin = io.pins.gpio18.into_pull_up_input(); // SDA reserved
    let button_19_pin = io.pins.gpio19.into_pull_up_input();
    let button_20_pin = io.pins.gpio20.into_pull_up_input();
    let button_21_pin = io.pins.gpio21.into_pull_up_input();
    let button_22_pin = io.pins.gpio22.into_pull_up_input();
    // let button_23_pin = io.pins.gpio23.into_pull_up_input(); // SCL reserved
    let button_24_pin = io.pins.gpio24.into_pull_up_input();
    let button_25_pin = io.pins.gpio25.into_pull_up_input();
    let button_26_pin = io.pins.gpio26.into_pull_up_input();
    let button_27_pin = io.pins.gpio27.into_pull_up_input();
    // let button_28_pin = io.pins.gpio28.into_pull_up_input(); // Not supported
    // let button_29_pin = io.pins.gpio29.into_pull_up_input(); // Not supported

    // let button_30_pin = io.pins.gpio30.into_pull_up_input(); // Not supported
    // let button_31_pin = io.pins.gpio31.into_pull_up_input(); // Not supported
    let button_32_pin = io.pins.gpio32.into_pull_up_input(); // Not supported


    loop {
        display.clear();
        Text::with_baseline("GPIOs example", Point::zero(), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();

        gpio_state(&mut display, 0, button_0_pin.is_low().unwrap());
        gpio_state(&mut display, 1, button_1_pin.is_low().unwrap());
        gpio_state(&mut display, 2, button_2_pin.is_low().unwrap());
        gpio_state(&mut display, 3, button_3_pin.is_low().unwrap());
        gpio_state(&mut display, 4, button_4_pin.is_low().unwrap());
        gpio_state(&mut display, 5, button_5_pin.is_low().unwrap());
        gpio_state(&mut display, 9, button_9_pin.is_low().unwrap());
        gpio_state(&mut display, 12, button_12_pin.is_low().unwrap());
        gpio_state(&mut display, 13, button_13_pin.is_low().unwrap());
        gpio_state(&mut display, 14, button_14_pin.is_low().unwrap());
        gpio_state(&mut display, 15, button_15_pin.is_low().unwrap());
        gpio_state(&mut display, 16, button_16_pin.is_low().unwrap());
        gpio_state(&mut display, 17, button_17_pin.is_low().unwrap());

        gpio_state(&mut display, 19, button_19_pin.is_low().unwrap());
        gpio_state(&mut display, 20, button_20_pin.is_low().unwrap());
        gpio_state(&mut display, 21, button_21_pin.is_low().unwrap());
        gpio_state(&mut display, 22, button_22_pin.is_low().unwrap());

        gpio_state(&mut display, 24, button_24_pin.is_low().unwrap());
        gpio_state(&mut display, 25, button_25_pin.is_low().unwrap());
        gpio_state(&mut display, 26, button_26_pin.is_low().unwrap());
        gpio_state(&mut display, 27, button_27_pin.is_low().unwrap());

        gpio_state(&mut display, 28, button_32_pin.is_low().unwrap());

        // display_state(&display, 0, button_0_pin.is_low().unwrap());s
        // if button_0_pin.is_low().unwrap() {
        //     Text::with_baseline("0", Point::new(0, 16), text_style, Baseline::Top)
        //         .draw(&mut display)
        //         .unwrap();
        // } else {
        //     Text::with_baseline("1", Point::new(0, 16), text_style, Baseline::Top)
        //     .draw(&mut display)
        //     .unwrap();

        // }

        // if button_right_pin.is_low().unwrap() {
        //     Text::with_baseline("Right", Point::new(60, 16), text_style, Baseline::Top)
        //         .draw(&mut display)
        //         .unwrap();
        // }

        display.flush().unwrap();
        delay.delay_ms(300u32);
    }
}
