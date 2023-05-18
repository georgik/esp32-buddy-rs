#![no_std]
#![no_main]
#![feature(c_variadic)]
#![feature(const_mut_refs)]

use embedded_io::blocking::*;
use embedded_svc::wifi::{
    AccessPointInfo, ClientConfiguration, ClientConnectionStatus, ClientIpStatus, ClientStatus,
    Configuration, Status, Wifi
};

use esp_backtrace as _;
use esp_println::{print, println};
use esp_wifi::wifi::utils::{create_network_interface};
use esp_wifi::wifi_interface::{timestamp, WifiError, Network};
use esp_wifi::{create_network_stack_storage, network_stack_storage};
use esp_wifi::{current_millis, initialize};
use hal::{i2c, IO, pac::Peripherals, prelude::*, timer::TimerGroup, Rtc, Delay, clock::{ClockControl, CpuClock}};
use smoltcp::wire::Ipv4Address;

use xtensa_lx_rt::entry;

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};

use ssd1306::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1306};

use heapless::String;

extern crate alloc;

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");

#[entry]
fn main() -> ! {
    esp_wifi::init_heap();

    let peripherals = Peripherals::take();

    let mut system = peripherals.DPORT.split();

    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock240MHz).freeze();

    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timg1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    initialize(timg1.timer0, peripherals.RNG, &clocks).unwrap();

    // Disable watchdog timers
    rtc.rwdt.disable();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

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

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    Text::with_baseline("WiFi example", Point::zero(), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

        display.flush().unwrap();

    let mut storage = create_network_stack_storage!(3, 8, 1);
    let ethernet = create_network_interface(network_stack_storage!(storage));
    let mut wifi_interface = esp_wifi::wifi_interface::Wifi::new(ethernet);

    println!("{:?}", wifi_interface.get_status());

    println!("Start Wifi Scan");

    let res: Result<(heapless::Vec<AccessPointInfo, 10>, usize), WifiError> =
        wifi_interface.scan_n();
    if let Ok((res, _count)) = res {
        for ap in res {
            println!("{:?}", ap);
        }
    }
    display.clear();
    Text::with_baseline("WiFi example\nScanning...", Point::zero(), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

    display.flush().unwrap();

    println!("Call wifi_connect");
    let client_config = Configuration::Client(ClientConfiguration {
        ssid: SSID.into(),
        password: PASSWORD.into(),
        ..Default::default()
    });
    let res = wifi_interface.set_configuration(&client_config);
    println!("wifi_connect returned {:?}", res);

    println!("{:?}", wifi_interface.get_capabilities());
    println!("{:?}", wifi_interface.get_status());

    // wait to get connected
    println!("Wait to get connected");

    loop {
        display.clear();
        Text::with_baseline("WiFi example\nConnecting", Point::zero(), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();
        display.flush().unwrap();

        for x in (60..75).step_by(5) {

            Text::new(".", Point::new(x, 16), text_style)
                .draw(&mut display)
                .unwrap();

            display.flush().unwrap();
            delay.delay_ms(1000u32);
        }

        if let Status(ClientStatus::Started(_), _) = wifi_interface.get_status() {
            display.clear();
            Text::with_baseline("WiFi example\nConnected.", Point::zero(), text_style, Baseline::Top)
                .draw(&mut display)
                .unwrap();
            display.flush().unwrap();
            break;
        }
    }
    println!("{:?}", wifi_interface.get_status());

    // wait for getting an ip address
    println!("Wait to get an ip address");

    loop {
        display.clear();
        Text::with_baseline("WiFi example\nConnected.\nGetting IP address", Point::zero(), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();
        display.flush().unwrap();

        wifi_interface.poll_dhcp().unwrap();

        wifi_interface
            .network_interface()
            .poll(timestamp())
            .unwrap();

        for x in (108..123).step_by(5) {

            Text::new(".", Point::new(x, 27), text_style)
                .draw(&mut display)
                .unwrap();

            display.flush().unwrap();
            delay.delay_ms(1000u32);
        }

        if let Status(
            ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(config))),
            _,
        ) = wifi_interface.get_status()
        {
            println!("got ip {:?}", config);
            use core::fmt::Write as FmtWrite;
            let mut ip_addr: heapless::String<256> = heapless::String::new();
            let bytes = config.ip.octets();
            write!(ip_addr,"{}.{}.{}.{}", bytes[0], bytes[1], bytes[2], bytes[3]).unwrap();
            display.clear();
            Text::with_baseline("WiFi example\nConnected.\nIP:", Point::zero(), text_style, Baseline::Top)
                .draw(&mut display)
                .unwrap();
            Text::new(&ip_addr, Point::new(21,28), text_style)
                .draw(&mut display)
                .unwrap();

            display.flush().unwrap();
            break;
        }
    }

    println!("Start busy loop on main");

    let mut network = Network::new(wifi_interface, current_millis);
    let mut socket = network.get_socket();

    loop {
        println!("Making HTTP request");
        socket.work();

        socket
            .open(Ipv4Address::new(142, 250, 185, 115), 80)
            .unwrap();

        socket
            .write(b"GET / HTTP/1.0\r\nHost: www.mobile-j.de\r\n\r\n")
            .unwrap();
        socket.flush().unwrap();

        let wait_end = current_millis() + 2 * 1000;
        loop {
            let mut buffer = [0u8; 512];
            if let Ok(len) = socket.read(&mut buffer) {
                let to_print = unsafe { core::str::from_utf8_unchecked(&buffer[..len]) };
                print!("{}", to_print);
            } else {
                break;
            }

            if current_millis() > wait_end {
                println!("Timeout");
                break;
            }
        }
        println!();

        socket.disconnect();

        let wait_end = current_millis() + 5 * 1000;
        while current_millis() < wait_end {
            socket.work();
        }
    }
}
