Esparrier
=========

Esparrier is a [Barrier](https://github.com/debauchee/barrier) client for ESP32S3.

This is a re-write of the original [Esparrier-IDF project](https://github.com/windoze/esparrier-idf), moving from `esp-idf-hal` to `esp-hal` baremetal infrastructure. Some functions are still missing, e.g. the clipboard support.

## How to build

1. Install Rust toolchain.
2. Install Rust ESP32 tools:
    * `espup` - https://github.com/esp-rs/espup
    * `ldproxy` - https://github.com/esp-rs/embuild
    * `cargo-espflash` - https://github.com/esp-rs/espflash
    * `espmonitor` - https://github.com/esp-rs/espmonitor
    * Install Rust ESP toolchain with `espup install`
3. Set environment variable for Rust ESP toolchain:
    * `source $HOME/export-esp.sh`
4. Build and flash:
    1. Set following environment variables:
        * `export WIFI_SSID="YOUR_WIFI_SSID"`
        * `export WIFI_PASSWORD="YOUR_WIFI_PASSWORD"`
        * `export BARRIER_SERVER="BARRIER_SERVER_IP:PORT"`
        * `export SCREEN_NAME="SCREEN_NAME"`
        * `export SCREEN_WIDTH="SCREEN_WIDTH"`
        * `export SCREEN_HEIGHT="SCREEN_HEIGHT"`
        * `export REVERSED_WHEEL="true to reverse the mouse wheel, false to use the default"`
    2. Put your board in the download mode, then build and flash with `cargo run --release`. On M5Atom S3 Lite, you need to hold the reset button until the green LED turns on, then release the button. And you need to press the reset button again after flashing to exit the download mode.

## Run

1. Configure Barrier server to accept the screen name you set in the environment variable `SCREEN_NAME`, and make sure you turn off the TLS.
2. Plug the board into the USB port.
3. The LED should be red on start, then turn blue when the board is connected to the WiFi, and finally turn dim yellow when the board is connected to the Barrier server.
4. When Barrier enters the screen, the LED turns bright green, and when Barrier leaves the screen, the LED turns dim yellow.
5. The board emulates a standard keyboard and an absolute mouse, it should work in any OS.
6. USB HID boot protocol is used, so you should be able to use the board as a USB keyboard/mouse in BIOS/EFI or even if the OS doesn't have a driver for it.

## Update Configurations

First, you need to install `esptool.py`, which can be installed with `pip install esptool`.

### Prepare and Update Configurations

1. Create a JSON file, refer to [config.json.example](config.json.example) for the format.
2. Put the board into the download mode, then use `esptool.py` to flash the NVS partition.
    ```bash
    esptool.py --chip esp32s3 --port /dev/ttyUSB0 write_flash 0x9000 /path/to/config.json
    ```
3. Exit the download mode and reset the board, the new configurations should be applied.

## Build for other ESP32S3 boards

* If there is a RGB LED (WS2812B) on the board, you can use `smartled` feature to enable the LED, and you need to set the environment `SMART_LED_PIN` to the correct pin number, on M5AtomS3/Lite, it's 35, on M5StampS3, it's 21.

* If there is a ordinary LED on the board, you can use `led` feature to enable it, and you need to set the environment `LED_PIN` to the correct pin number.

* Do not enable more than one of above features, the program won't compile.

* If none of above features is enabled, the indicator function is disabled.

* The program will output log to the UART0 by default, you can use `espmonitor` to monitor the log. If your board doesn't have separated UART0 port, you can disable the default features, this will disable the USB HID function, and you'll be able to see logs from USB OTG/J-TAG port. This is useful for debugging codes not related to USB HID.

## NOTES:

**WARNING**: This program is only for testing purpose. It is not a complete implementation of Barrier client. There could be a lot of bugs and missing features. It has no concept of security, neither on the WiFi nor on the USB. It is not recommended to use it in anywhere but a private environment.

* This code is developed and tested on [M5Atom S3 Lite](https://docs.m5stack.com/en/core/AtomS3%20Lite), other ESP32S3 boards may not work, or you need to change the code.
* A board with external antenna is strongly recommended, the ESP32S3 support 2.4G WiFi only and this band is really crowded, you may experience jittering and lagging if the wireless connection is not stable.
* The code doesn't work on ESP8266/ESP32/ESP32C3 because they don't have required USB features.
* It doesn't support TLS, so you must run Barrier server without TLS.
* The mouse is configured to the absolute mode, you must set the correct screen resolution before building, otherwise the mouse may not work properly.
* Clipboard, file transfer, and cross-screen drag and drop are not supported due to the technical limitation, there is no way a standard USB HID device can do that, maybe an auxiliary app running on the host can help but I still don't have clear idea.
* Auto-switching doesn't work properly unless you set the screen size correctly, otherwise you may need to configure hotkey on the Barrier server to switch screens manually.
* Frequently connect/disconnect may cause the board fail to connect to the WiFi and/or Barrier server, you may need to power off the board and wait for a while before trying again.
* In theory the board should be working with [InputLeap](https://github.com/input-leap/input-leap) server as well but I've never tested it.
* The USB VID/PID are randomly picked and not registered, so you may need to change the code to use your own VID/PID.
* The USB remote wakeup may not work because the standard forbids a suspended device consume too much current but this program needs much more than the standard says to keep Wi-Fi connected. I still haven't figured out how to keep the program running with the current <2.5mA. Of course you can choose a board with external power source such as a battery, but it seems to be an overkill.
* The program can accept inputs only **after** the board successfully connects to the WiFi and Barrier server, it may be too late to use the board as a USB keyboard/mouse in BIOS/EFI, some main board that has always-on USB ports may work, but I haven't tested it, or you can use a USB hub that can supply power even if the host is off.
* The watchdog will reset the board if it doesn't receive heartbeat from the Barrier server, or the program itself runs out of control and doesn't process the heartbeat, for the number of seconds defined in `WATCHDOG_TIMEOUT` environment variable. The default watchdog timeout is 15 seconds, as the default Barrier heartbeat interval is 5 seconds, you may need to change the watchdog timeout if the Barrier server has a long heartbeat interval.

## TODO:

- [x] Support media keys
- [x] Re-configure without rebuilding
- [x] Support other ESP32S3 boards
- [ ] Support Mac special keys
- [ ] Support TLS
- [ ] NVS encryption
- [ ] OTA update
- [ ] Support clipboard, maybe with a separate app running on the host to handle the clipboard data

## Licenses and Copyrights

* This project is released under MIT license.
* The `esp_hal_smartled.rs` file is taken from (esp-hal-community repo)[https://github.com/esp-rs/esp-hal-community], which is licensed under MIT License and Apache License, version 2.0.
* Some code snippets in the `gentable.c` were taken from (Barrier repo)[https://github.com/debauchee/barrier] and licensed under GPLv2. The main project only uses it's output, not the code itself.