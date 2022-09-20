#![no_std]
#![no_main]

// Based on: https://github.com/espressif/esp-mdf/tree/master/examples/development_kit/buddy

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};

use embedded_io::blocking::*;
use embedded_svc::wifi::{
    AccessPointInfo, ClientConfiguration, ClientConnectionStatus, ClientIpStatus, ClientStatus,
    Configuration, Status, Wifi,
};

use ssd1306::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1306};
use esp32_hal::{clock::ClockControl, i2c, IO, pac::Peripherals, prelude::*, timer::TimerGroup, Rtc};
use esp_backtrace as _;
use xtensa_lx_rt::entry;

use esp_println::{print, println};
use esp_wifi::wifi::utils::{create_network_interface, Network};
use esp_wifi::wifi_interface::{timestamp, WifiError};
use esp_wifi::{create_network_stack_storage, network_stack_storage};
use esp_wifi::{current_millis, initialize};

use smoltcp::wire::Ipv4Address;

extern crate alloc;

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");

#[entry]
fn main() -> ! {
    esp_wifi::init_heap();

    let peripherals = Peripherals::take().unwrap();
    let mut system = peripherals.DPORT.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let mut storage = create_network_stack_storage!(3, 8, 1);
    let ethernet = create_network_interface(network_stack_storage!(storage));
    let mut _wifi_interface = esp_wifi::wifi_interface::Wifi::new(ethernet);

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    let mut wdt1 = timer_group1.wdt;

    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

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

    Text::with_baseline("WiFi example", Point::zero(), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

    // Text::with_baseline("Hello Rust!", Point::new(0, 16), text_style, Baseline::Top)
    //     .draw(&mut display)
    //     .unwrap();


    println!("{:?}", _wifi_interface.get_status());

    println!("Start Wifi Scan");
    let res: Result<(heapless::Vec<AccessPointInfo, 10>, usize), WifiError> =
        _wifi_interface.scan_n();
    if let Ok((res, _count)) = res {
        for ap in res {
            println!("{:?}", ap);
        }
    }

    println!("Call wifi_connect");
    let client_config = Configuration::Client(ClientConfiguration {
        ssid: SSID.into(),
        password: PASSWORD.into(),
        ..Default::default()
    });

    Text::new("Connected.", Point::new(0, 20), text_style)
        .draw(&mut display)
        .unwrap();

    let res = _wifi_interface.set_configuration(&client_config);
    //println!("wifi_connect returned {:?}", res);

    //println!("{:?}", _wifi_interface.get_capabilities());
    //println!("{:?}", _wifi_interface.get_status());
    
    display.flush().unwrap();

    loop {}
}
