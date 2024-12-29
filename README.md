Esparrier
=========
[![en](https://img.shields.io/badge/lang-en-blue.svg)](https://github.com/windoze/esparrier/blob/main/README.md)
[![zh-cn](https://img.shields.io/badge/lang-zh--cn-green.svg)](https://github.com/windoze/esparrier/blob/main/README.zh-cn.md)
![CI](https://github.com/windoze/esparrier/actions/workflows/ci.yaml/badge.svg)
[![GitHub release (latest by date)](https://img.shields.io/github/v/release/windoze/esparrier?logo=GitHub)](https://github.com/windoze/esparrier/releases)

Esparrier is a [Barrier](https://github.com/debauchee/barrier) client for ESP32S3.

This is a re-write of the original [Esparrier-IDF project](https://github.com/windoze/esparrier-idf), moving from `esp-idf-hal` to `esp-hal` baremetal infrastructure.

## How to build

1. Install Rust toolchain.
2. Install Rust ESP32 tools:
    * `espup` - https://github.com/esp-rs/espup
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

First, you need to install `esptool.py`, which can be installed with `pip install esptool`. Refer to the [official documentation](https://docs.espressif.com/projects/esptool/en/latest/esp32/installation.html) for more information.

### Prepare and Update Configurations

1. Create a JSON file, refer to [config.json.example](config.json.example) for the format.
2. Put the board into the download mode, then use `esptool.py` to flash the NVS partition.
    ```bash
    # Erase the NVS partition
    esptool.py --chip esp32s3 --port /dev/ttyACM0 write_flash 0x9000 zero.bin
    # Write the configurations
    esptool.py --chip esp32s3 --port /dev/ttyACM0 write_flash 0x9000 /path/to/config.json
    ```
3. Exit the download mode and reset the board, the new configurations should be applied.

## Clipboard

The program now has limited support of the clipboard when the feature `clipboard` is enabled. This feature requires your board to have an user button, and the button should be connected to the GPIO pin defined in the environment variable `PASTE_BUTTON_PIN`, on M5AtomS3 Lite, it is PIN 41. The button should be active low, i.e. the pin should be pulled up to high level, and the button should pull down the pin to low level when pressed.

First you need to activate other screen and copy something into the clipboard, then switch to the screen connected to the board.

When the screen is activated, the board receives the clipboard content sent by the Barrier server, **keeps the first 1024 characters of the plain text format and discard everything else**.

Then you can "paste" the text by pressing the button on the board, the board will convert the text into a sequence of keystrokes, and send them to the computer. All characters except the visible ASCII codes will be discarded as they cannot be directly mapped to USB HID key codes, or they may have special meaning that can mess up things.

The program cannot "copy" content to the clipboard.

NOTE: When you copied a large amount of text or big image from other screen then moved into the screen connected to the board, the board may stuck for a while, this is because the board is trying to discard the clipboard content. Even it will not parse and hold the whole content, still it needs to receive the whole content from the Barrier server as there is no way to skip a chunk in the middle of a TCP stream without actually reading it. But the board should resume operation after few seconds and it will not repeatedly process the same clipboard content if you move out and move in again.

## Build for other ESP32S3 boards

* The release page provides pre-built binaries for some ESP32S3 boards, you can use them directly if your board is listed.
    * Generic ESP32S3 boards with native USB-OTG port, the indicator feature is unavailable, and the clipboard feature is disabled as well.
    * [M5Atom S3 Lite](https://docs.m5stack.com/en/core/AtomS3%20Lite), the SmartLED feature and the clipboard feature are enabled.
    * [M5Atom S3](https://docs.m5stack.com/en/core/AtomS3), the clipboard feature are enabled, and the indicator shows emoji animations on the built-in LCD screen.
    * [Seeed Studio XIAO ESP32S3](https://wiki.seeedstudio.com/xiao_esp32s3_getting_started/), the led is used as the indicator, and the clipboard feature is disabled due to the lack of a user button.

* It's recommended to erase the flash before the first time flashing the binary to the board, you can do this with `esptool.py` or `cargo-espflash`:
    ```bash
    # With cargo-espflash
    cargo espflash erase-flash --chip esp32s3 --port /dev/ttyACM0
    
    # With esptool.py
    esptool.py --chip esp32s3 --port /dev/ttyACM0 erase_flash
    ```

* If there is a RGB LED (WS2812B) on the board, you can use `smartled` feature to enable the LED, and you need to set the environment `SMART_LED_PIN` to the correct pin number, on M5AtomS3/Lite, it's 35, on M5StampS3, it's 21.
    * E.g. to build and flash the binary for ESP32-S3-DevKitC-1:
        ```bash
        SMART_LED_PIN=38 cargo run --release --features smartled
        ```

* If there is an ordinary LED on the board, you can use `led` feature to enable it, and you need to set the environment `LED_PIN` to the correct pin number.

* Do not enable more than one of above features, the program won't compile.

* If none of above features is enabled, the indicator function is disabled.

* The program will output log to the UART0 by default, you can use `espmonitor` to monitor the log. If your board doesn't have separated UART0 port, you can disable the default features, this will disable the USB HID function, and you'll be able to see logs from USB OTG/J-TAG port. This is useful for debugging codes not related to USB HID.

## Use pre-built binaries

**NOTE**: Using pre-built binaries is not recommended, because it cannot utilize the full potential of the board, and you may need to change the code to fit your board. So you should build the binary yourself when possible.

1. Install `esptool.py` as described in the previous section. You don't need to install Rust toolchain and any other ESP32 tools.

2. Download the binary from the [releases page](https://github.com/windoze/esparrier/releases).

3. Extract the binary from the archive. There are 4 pre-built binaries in the archive, choose the one that fits your board.
    * `esparrier.bin` - For generic ESP32S3 boards with native USB-OTG port, the indicator feature is unavailable.
    * `esparrier-m5atoms3-lite.bin` - For [M5Atom S3 Lite](https://docs.m5stack.com/en/core/AtomS3%20Lite).
    * `esparrier-m5atoms3.bin` - For [M5Atom S3](https://docs.m5stack.com/en/core/AtomS3).
    * `esparrier-xiao-esp32s3.bin` - For [Seeed Studio XIAO ESP32S3](https://wiki.seeedstudio.com/xiao_esp32s3_getting_started/).

4. Prepare the `config.json` file as described in the previous section.

5. Put the board into the download mode, then flash the binary and config to the board. Note the USB device name may vary, you may need to change it to the correct one. On most Linux systems, the device name is `/dev/ttyACMx`, where `x` is a number, you can find the correct device name by running `ls /dev/ttyACM*`.
    ```bash
    # Erase the whole flash
    esptool.py --chip esp32s3 --port /dev/ttyACM0 erase_flash
    # Write the binary and config
    esptool.py --chip esp32s3 --port /dev/ttyACM0 write_flash 0x10000 /path/to/esparrier.bin 0x9000 /path/to/config.json
    ```

6. Exit the download mode and reset the board, you should see the new USB HID device on your host.

## NOTES:

**WARNING**: This program is only for testing purpose. It is not a complete implementation of Barrier client. There could be a lot of bugs and missing features. It has no concept of security, neither on the WiFi nor on the USB. It is not recommended to use it in anywhere but a private environment.

* This code is developed and tested on [M5Atom S3 Lite](https://docs.m5stack.com/en/core/AtomS3%20Lite), other ESP32S3 boards may not work, or you need to change the code to fit your board.
* A board with external antenna is strongly recommended, the ESP32S3 supports 2.4G WiFi only and this band is really crowded, you may experience jittering and lagging if the wireless connection is not stable.
* The code doesn't work on ESP8266/ESP32/ESP32C3 because they don't have required USB features, ESP32S2 may work with adaptation but it's not tested.
* It doesn't support TLS, so you must run Barrier server without TLS.
* The mouse is configured to the absolute mode, you must set the correct screen resolution before building, otherwise the mouse may not work properly.
* Clipboard, file transfer, and cross-screen drag and drop are not supported due to the technical limitation, there is no way a standard USB HID device can do that, maybe an auxiliary app running on the host can help but I still don't have clear idea.
* Auto-switching doesn't work properly unless you set the screen size correctly, otherwise you may need to configure hotkey on the Barrier server to switch screens manually.
* Frequently connect/disconnect may cause the board fail to connect to the WiFi and/or Barrier server, you may need to power off the board and wait for a while before trying again.
* In theory the board should be working with [InputLeap](https://github.com/input-leap/input-leap) server as well but I've never tested it.
* The USB VID/PID are randomly picked and not registered, you are not authorized to produce and sell USB devices using these VID/PID, so you may need to change the code to use your own VID/PID if you have any business purpose.
* The USB remote wakeup may not work because the standard forbids a suspended device consume too much current but this program needs much more than the standard says to keep Wi-Fi connected. I still haven't figured out how to keep the program running with the current <2.5mA. Of course you can choose a board with external power source such as a battery, but it seems to be an overkill.
* The program can accept inputs only **after** the board successfully connects to the WiFi and Barrier server, it may be too late to use the board as a USB keyboard/mouse in BIOS/EFI, some main board that has always-on USB ports may work, but I haven't tested it, or you can use a USB hub that can supply power even if the host is off.
* The watchdog will reset the board if it doesn't receive heartbeat from the Barrier server, or the program itself runs out of control and doesn't process the heartbeat, for the number of seconds defined in `WATCHDOG_TIMEOUT` environment variable. The default watchdog timeout is 15 seconds, as the default Barrier heartbeat interval is 5 seconds, you may need to change the watchdog timeout if the Barrier server has a long heartbeat interval.
* The built-in clipboard sharing function is incomplete, but it can be achieved with a [ClipSync app](https://github.com/windoze/clip-sync), or the OS's built-in clipboard sharing function, such as [Clip Sync on Windows](https://support.microsoft.com/en-us/windows/about-the-clipboard-in-windows-c436501e-985d-1c8d-97ea-fe46ddf338c6) or [Universal Clipboard on macOs](https://support.apple.com/guide/mac-help/copy-and-paste-between-devices-mchl70368996/mac).

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

* This project is released under [MIT license](LICENSE).
* The `esp_hal_smartled.rs` file is taken from [esp-hal-community repo](https://github.com/esp-rs/esp-hal-community), which is licensed under [MIT License](https://github.com/esp-rs/esp-hal-community/blob/main/LICENSE-MIT) and [Apache License, version 2.0](https://github.com/esp-rs/esp-hal-community/blob/main/LICENSE-APACHE).
* Some code snippets in the `gentable.c` were taken from [Barrier repo](https://github.com/debauchee/barrier) and licensed under [GPLv2](https://github.com/debauchee/barrier/blob/master/LICENSE). The main project only uses it's output, not the code itself, so it's not bound by the GPL license.
* Animated emoji images are taken from [Google Font Project](https://googlefonts.github.io/noto-emoji-animation/), which is licensed under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).