#![no_std]
#![no_main]

extern crate alloc;
use core::mem::MaybeUninit;
use esp_backtrace as _;
use esp_println::println;
use hal::{
    clock::ClockControl, clock::CpuClock, i2c, peripherals::Peripherals, prelude::*, Delay, IO,
};

use esp_wifi::{initialize, EspWifiInitFor};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

use hal::{timer::TimerGroup, Rng};

use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use embedded_svc::ipv4::Interface;
use embedded_svc::wifi::{AccessPointInfo, ClientConfiguration, Configuration, Wifi};
use esp_wifi::current_millis;
use esp_wifi::wifi::{utils::create_network_interface, WifiStaDevice};
use esp_wifi::wifi::{WifiError, WifiMode};
use esp_wifi::wifi_interface::WifiStack;
use lexical_core;
use smoltcp::iface::SocketStorage;
use smoltcp::wire::Ipv4Address;

use embedded_svc::io::{Read, Write};

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

fn init_heap() {
    const HEAP_SIZE: usize = 5 * 1024;
    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

    unsafe {
        ALLOCATOR.init(HEAP.as_mut_ptr() as *mut u8, HEAP_SIZE - 1024);
    }
}

const NTP_VERSION: u8 = 0b00100011; // NTP version 4, mode 3 (client)
const NTP_MODE: u8 = 0b00000011;
const NTP_PACKET_SIZE: usize = 48;
const NTP_TIMESTAMP_DELTA: u64 = 2_208_988_800; // 70 years in seconds (since 01.01.1900)
const TIMESTAMP_LEN: usize = 10;
const UNIXTIME_LEN: usize = 8;

fn is_char_in_str(s: &str, c: char) -> bool {
    for byte in s.as_bytes() {
        if *byte == c as u8 {
            return true;
        }
    }
    false
}

type NtpRequest = [u8; NTP_PACKET_SIZE];

pub fn new_request(timestamp: u64) -> NtpRequest {
    let mut buf: [u8; 48] = [0u8; 48];

    // Set Leap Indicator (LI), Protocol Version (VN), and Mode (3 = Client)
    buf[0] = 0b00_011_011;

    // Set Stratum (0 = unspecified)
    buf[1] = 0;

    // Set Poll Interval (4 = 16 seconds)
    buf[2] = 4;

    // Set Precision (-6 = 15.26 microseconds)
    buf[3] = 0xFA;

    // Set Root Delay
    buf[4] = 0;
    buf[5] = 0;
    buf[6] = 0;
    buf[7] = 0;

    // Set Root Dispersion
    buf[8] = 0;
    buf[9] = 0;
    buf[10] = 0;
    buf[11] = 0;

    // Set Reference Identifier (unspecified)
    buf[12] = 0;
    buf[13] = 0;
    buf[14] = 0;
    buf[15] = 0;

    // Set Originate Timestamp to current time
    let secs = timestamp + 2_208_988_800;
    let frac =
        ((timestamp % 1_000_000_000) as f64 / 1_000_000_000.0) * ((2.0 as u32).pow(32) as f64);
    let frac = frac as u32;
    buf[16..24].copy_from_slice(&secs.to_be_bytes());
    buf[24..32].copy_from_slice(&frac.to_be_bytes());

    // Leave Transmit Timestamp and Receive Timestamp as 0

    buf
}

fn find_unixtime(response: &[u8]) -> Option<u64> {
    // Convert the response to a string slice
    let response_str = core::str::from_utf8(response).ok()?;

    // Look for the "unixtime" key in the response
    let unixtime_key = b"\"unixtime\":";
    if let Some(start) = response_str.find(core::str::from_utf8(unixtime_key).ok()?) {
        // Find the start of the number (skipping the key and any potential spaces)
        let number_start = start + unixtime_key.len();
        let number_end = response_str[number_start..]
            .find(|c: char| !c.is_digit(10) && c != ' ')
            .map_or(response_str.len(), |end| number_start + end);

        // Parse the number
        response_str[number_start..number_end].parse().ok()
    } else {
        None
    }
}

fn timestamp_to_hms(timestamp: u64) -> (u64, u64, u64) {
    let seconds_per_minute = 60;
    let minutes_per_hour = 60;
    let hours_per_day = 24;
    let seconds_per_hour = seconds_per_minute * minutes_per_hour;
    let seconds_per_day = seconds_per_hour * hours_per_day;

    let hours = (timestamp % seconds_per_day) / seconds_per_hour;
    let minutes = (timestamp % seconds_per_hour) / seconds_per_minute;
    let seconds = timestamp % seconds_per_minute;

    (hours, minutes, seconds)
}

#[entry]
fn main() -> ! {
    let mut rx_buffer = [0u8; 1536];
    let mut tx_buffer = [0u8; 1536];
    let mut buffer = [0u8; 4096];
    let mut socket_set_entries: [SocketStorage; 5] = Default::default();

    init_heap();
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();

    // let clocks = ClockControl::max(system.clock_control).freeze();
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock240MHz).freeze();
    let mut delay = Delay::new(&clocks);

    // setup logger
    // To change the log_level change the env section in .cargo/config.toml
    // or remove it and set ESP_LOGLEVEL manually before running cargo run
    // this requires a clean rebuild because of https://github.com/rust-lang/cargo/issues/10358
    esp_println::logger::init_logger_from_env();
    log::info!("Logger is setup");
    println!("Hello world!");
    let timer = TimerGroup::new(peripherals.TIMG1, &clocks).timer0;
    let init = initialize(
        EspWifiInitFor::Wifi,
        timer,
        Rng::new(peripherals.RNG),
        system.radio_clock_control,
        &clocks,
    )
    .unwrap();
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let sda = io.pins.gpio18;
    let scl = io.pins.gpio23;

    let i2c = i2c::I2C::new(peripherals.I2C0, sda, scl, 100u32.kHz(), &clocks);

    let interface = I2CDisplayInterface::new(i2c);

    let wifi = peripherals.WIFI;
    let (iface, device, mut controller, sockets) =
        create_network_interface(&init, wifi, WifiStaDevice, &mut socket_set_entries).unwrap();

    let wifi_stack = WifiStack::new(iface, device, sockets, current_millis);

    let client_config = Configuration::Client(ClientConfiguration {
        ssid: SSID.into(),
        password: PASSWORD.into(),
        ..Default::default()
    });
    let mut display = Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();
    let text_style2 = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(BinaryColor::On)
        .build();

    Text::with_baseline("Initializing...", Point::zero(), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();
    display.flush().unwrap();

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

    let mut socket = wifi_stack.get_socket(&mut rx_buffer, &mut tx_buffer);

    println!("Making HTTP request");
    socket.work();
    println!("Minimum free heap size: {} bytes", ALLOCATOR.free());

    match socket.open(
        smoltcp::wire::IpAddress::Ipv4(Ipv4Address::new(213, 188, 196, 246)),
        80,
    ) {
        Ok(_) => println!("Socket opened"),
        Err(e) => println!("Error opening socket: {:?}", e),
    }

    socket
        .write(
            "GET /api/timezone/Europe/Prague HTTP/1.1\r\nHost: worldtimeapi.org\r\n\r\n".as_bytes(),
        )
        .unwrap();
    socket.flush().unwrap();

    let wait_end = current_millis() + 20 * 1000;
    let mut timestamp: u64 = 0;

    let wait_end = current_millis() + 20 * 1000;
    println!("Minimum free heap size: {} bytes", ALLOCATOR.free());
    let mut total_size = 0;
    let mut total_size = 0usize;

    loop {
        if total_size >= buffer.len() {
            // Buffer is full
            println!("Buffer is full, processed {} bytes", total_size);
            // Here you might want to process the buffer and then clear it
            // ... (process buffer)
            total_size = 0; // Reset total_size if you wish to reuse the buffer
                            // continue; // Optionally continue reading after processing
            break; // or break if you're done
        }

        let buffer_slice = &mut buffer[total_size..]; // Slice the buffer from the current total_size to the end
        match socket.read(buffer_slice) {
            Ok(0) => {
                // The connection has been closed by the peer
                println!("Connection closed, total read size: {}", total_size);
                break;
            }
            Ok(len) => {
                println!("Read {} bytes", len);
                total_size += len;
                // buffer[..total_size] now contains the data read in this iteration
            }
            Err(e) => {
                // Handle the error (e.g., by breaking the loop or retrying)
                println!("Failed to read from socket: {:?}", e);
                break;
            }
        }
    }

    socket.disconnect();

    let wait_end = current_millis() + 5 * 1000;
    while current_millis() < wait_end {
        socket.work();
    }
    let to_print = unsafe { core::str::from_utf8_unchecked(&buffer[..total_size]) };
    println!("{}", to_print);
    if let Some(timestamp) = find_unixtime(&buffer[..total_size]) {
        println!("Timestamp: {}", timestamp);
        let mut timestamp = timestamp;
        timestamp += 60 * 60;
        loop {
            let (hours, minutes, seconds) = timestamp_to_hms(timestamp);
            let text = alloc::format!("{:02}:{:02}:{:02}", hours, minutes, seconds);

            display.clear();
            Text::with_baseline(text.as_str(), Point::zero(), text_style2, Baseline::Top)
                .draw(&mut display)
                .unwrap();
            display.flush().unwrap();

            println!("Loop...");
            delay.delay_ms(972u32); // use 972ms to get 1s delay, accounting also for rest of the code execution
            timestamp += 1;
        }
    } else {
        println!("Failed to find or parse the 'unixtime' field.");
    }
    println!("Timestamp: {}", timestamp);

    loop {}
}
