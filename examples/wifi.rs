#![no_std]
#![no_main]

use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};

use esp_backtrace as _;
use esp_println::logger::init_logger;
use esp_println::println;
use esp_wifi::wifi_interface::WifiStack;
use esp_wifi::{
    init,
    wifi::{
        utils::create_network_interface, AccessPointInfo, ClientConfiguration, Configuration,
        WifiError, WifiStaDevice,
    },
    EspWifiInitFor,
};
use hal::{
    gpio::Io,
    i2c,
    prelude::*,
    rng::Rng,
    time::{self},
    timer::timg::TimerGroup,
};
use smoltcp::iface::SocketStorage;

const SSID: &str = "SSID"; // env!("SSID");
const PASSWORD: &str = "PASSWORD"; // env!("PASSWORD");

#[entry]
fn main() -> ! {
    init_logger(log::LevelFilter::Info);

    esp_alloc::heap_allocator!(72 * 1024);

    let peripherals = hal::init(hal::Config::default());

    let timer = TimerGroup::new(peripherals.TIMG1).timer0;

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

    display.clear();
    Text::with_baseline("WiFi example", Point::zero(), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

    display.flush().unwrap();

    let rng = Rng::new(peripherals.RNG);
    let init = init(EspWifiInitFor::Wifi, timer, rng, peripherals.RADIO_CLK)
        .map_err(|e| println!("Failed to initialize wifi {:?}", e))
        .unwrap();

    let wifi = peripherals.WIFI;
    let mut socket_set_entries: [SocketStorage; 5] = Default::default();
    let (iface, device, mut controller, sockets) =
        create_network_interface(&init, wifi, WifiStaDevice, &mut socket_set_entries).unwrap();

    let now = || time::now().duration_since_epoch().to_millis();

    let wifi_stack = WifiStack::new(iface, device, sockets, now);

    let client_config = Configuration::Client(ClientConfiguration {
        ssid: SSID.try_into().unwrap(),
        password: PASSWORD.try_into().unwrap(),
        ..Default::default()
    });
    let res = controller.set_configuration(&client_config);
    println!("wifi_set_configuration returned {:?}", res);

    controller.start().unwrap();
    println!("is wifi started: {:?}", controller.is_started());

    println!("Start Wifi Scan");
    let res: Result<(heapless::Vec<AccessPointInfo, 10>, usize), WifiError> = controller.scan_n();
    if let Ok((res, _count)) = res {
        for ap in res {
            println!("{:?}", ap);
        }
    }

    println!("{:?}", controller.get_capabilities());
    println!("wifi_connect {:?}", controller.connect());

    // wait to get connected
    println!("Wait to get connected");
    display.clear();
    Text::with_baseline(
        "WiFi example\nWait to get connected",
        Point::zero(),
        text_style,
        Baseline::Top,
    )
    .draw(&mut display)
    .unwrap();
    display.flush().unwrap();

    loop {
        let res = controller.is_connected();
        match res {
            Ok(connected) => {
                if connected {
                    break;
                }
            }
            Err(err) => {
                println!("{:?}", err);
                loop {}
            }
        }
    }
    println!("{:?}", controller.is_connected());

    // wait for getting an ip address
    println!("Wait to get an ip address");
    loop {
        wifi_stack.work();

        if wifi_stack.is_iface_up() {
            println!("got ip {:?}", wifi_stack.get_ip_info());
            use core::fmt::Write as FmtWrite;
            let mut ip_addr: heapless::String<256> = heapless::String::new();
            let bytes = wifi_stack.get_ip_info().unwrap().ip.octets();
            write!(
                ip_addr,
                "{}.{}.{}.{}",
                bytes[0], bytes[1], bytes[2], bytes[3]
            )
            .unwrap();
            display.clear();
            Text::with_baseline(
                "WiFi example\nConnected.\nIP:",
                Point::zero(),
                text_style,
                Baseline::Top,
            )
            .draw(&mut display)
            .unwrap();
            Text::new(&ip_addr, Point::new(21, 28), text_style)
                .draw(&mut display)
                .unwrap();
            display.flush().unwrap();
            break;
        }
    }

    println!("Start busy loop on main");

    let mut rx_buffer = [0u8; 1536];
    let mut tx_buffer = [0u8; 1536];
    let _socket = wifi_stack.get_socket(&mut rx_buffer, &mut tx_buffer);

    loop {}
}
