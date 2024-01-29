# ESP32-Buddy-RS

Rust Bare Metal implementation of ESP-Buddy based on https://github.com/espressif/esp-mdf/tree/master/examples/development_kit/buddy

[![Wokwi](https://img.shields.io/endpoint?url=https%3A%2F%2Fwokwi.com%2Fbadge%2Fclick-to-simulate.json)](https://wokwi.com/projects/381376751746803713)

![ESP32-Buddy](esp32-buddy-rust-display.jpg)

[![Open ESP32 in Gitpod](https://gitpod.io/button/open-in-gitpod.svg)](https://gitpod.io/github.com/georgik/esp32-buddy-rs/)

## Quick start in Wokwi

Copy the code from examples to `main.rs` and run `./scripts/run-wokwi.sh`.
Once the build is complete, click the link in terminal to open Wokwi simulator.

Flash the device using web flasher by command `./scripts/flash.sh`

## How to use examples locally

Run the base application:

```
cargo run --release
```

This is mapped by `.cargo/config.toml` to command:

```
cargo espflash flash --release --monitor
```

Run animation example with:

```
cargo run --release --example animation
```

### Available examples

- animation - moving letters
```
cargo run --release --example animation
```
- blinky - blink LED - not working - blocked by:
   - https://github.com/georgik/esp32-buddy-rs/issues/1
   - https://github.com/esp-rs/esp-hal/issues/855
   - https://github.com/bjoernQ/esp-hal/pull/1/files

- buttons - display state of buttons
```
cargo run --release --example buttons
```
- clock - use Wi-Fi to acquire timestamp from NTP server and display time
```
export SSID="replace_by_ssid"
export PASSWORD="replace_by_password"
cargo run --release --example clock
```
- gpio - display state of GPIOs
```
cargo run --release --example gpio
```
- rainbow - iterate over HUE and display value - not working - blocked by:
   - https://github.com/georgik/esp32-buddy-rs/issues/1
   - https://github.com/esp-rs/esp-hal/issues/855
   - https://github.com/bjoernQ/esp-hal/pull/1/files
- snow - snowflakes falling
```
cargo run --release --example snow
```
- temperature - display temperature and humidity
```
cargo run --release --example temperature
```
- wifi - connect to Wi-Fi
```
export SSID="replace_by_ssid"
export PASSWORD="replace_by_password"
cargo run --release --example wifi
```


### Wokwi Simulation
When using a custom Wokwi project, please change the `WOKWI_PROJECT_ID` in
`run-wokwi.sh`. If no project id is specified, a DevKit for esp32 will be
used.

- Terminal approach:

    ```
    scripts/run-wokwi.sh [debug | release]
    ```
    > If no argument is passed, `release` will be used as default

- UI approach:

    The default test task is already set to build the project, and it can be used
    in VS Code and Gitpod:
    - From the [Command Palette](https://code.visualstudio.com/docs/getstarted/userinterface#_command-palette) (`Ctrl-Shift-P` or `Cmd-Shift-P`) run the `Tasks: Run Test Task` command
    - With `Ctrl-Shift-,` or `Cmd-Shift-,`
        > **Note**
        >
        > This Shortcut is not available in Gitpod by default.
    - From the [Command Palette](https://code.visualstudio.com/docs/getstarted/userinterface#_command-palette) (`Ctrl-Shift-P` or `Cmd-Shift-P`) run the `Tasks: Run Task` command and
    select `Build & Run Wokwi`.
    - From UI: Press `Build & Run Wokwi` on the left side of the Status Bar.

#### Debuging with Wokwi

Wokwi offers debugging with GDB.

- Terminal approach:
    ```
    $HOME/.espressif/tools/xtensa-esp32-elf/esp-2021r2-patch3-8.4.0/xtensa-esp32-elf/bin/xtensa-esp32-elf-gdb target/xtensa-esp32-espidf/debug/esp_buddy_rs -ex "target remote localhost:9333"
    ```

    > [Wokwi Blog: List of common GDB commands for debugging.](https://blog.wokwi.com/gdb-avr-arduino-cheatsheet/?utm_source=urish&utm_medium=blog)
- UI approach:
    1. Run the Wokwi Simulation in `debug` profile
    2. Go to `Run and Debug` section of the IDE (`Ctrl-Shift-D or Cmd-Shift-D`)
    3. Start Debugging by pressing the Play Button or pressing `F5`
    4. Choose the proper user:
        - `esp` when using VS Code or GitHub Codespaces
        - `gitpod` when using Gitpod
