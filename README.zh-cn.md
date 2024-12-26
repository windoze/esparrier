Esparrier
=========

[![en](https://img.shields.io/badge/lang-en-blue.svg)](https://github.com/windoze/esparrier/blob/main/README.md)

Esparrier 是一个适用于 ESP32S3 的 [Barrier](https://github.com/debauchee/barrier) 客户端。

这是对原有 [Esparrier-IDF 项目](https://github.com/windoze/esparrier-idf) 的重写，从 `esp-idf-hal` 迁移到 `esp-hal`。一些功能尚未完全实现，例如剪贴板支持。

## 如何构建

1. 安装 Rust 工具链。
2. 安装 Rust ESP32 工具：
    * `espup` - https://github.com/esp-rs/espup
    * `cargo-espflash` - https://github.com/esp-rs/espflash
    * `espmonitor` - https://github.com/esp-rs/espmonitor
    * 使用 `espup install` 安装 Rust ESP 工具链
3. 为 Rust ESP 工具链设置环境变量：
    * `source $HOME/export-esp.sh`
4. 构建和烧录：
    1. 设置以下环境变量：
        * `export WIFI_SSID="YOUR_WIFI_SSID"`
        * `export WIFI_PASSWORD="YOUR_WIFI_PASSWORD"`
        * `export BARRIER_SERVER="BARRIER_SERVER_IP:PORT"`
        * `export SCREEN_NAME="SCREEN_NAME"`
        * `export SCREEN_WIDTH="SCREEN_WIDTH"`
        * `export SCREEN_HEIGHT="SCREEN_HEIGHT"`
        * `export REVERSED_WHEEL="true 反转鼠标滚轮, false 使用默认值"`
    2. 将开发板置于下载模式，然后使用 `cargo run --release` 构建和烧录。在 M5Atom S3 Lite 上，需要按住复位按钮直到绿色 LED 亮起，然后松开按钮。烧录后需要再次按下复位按钮以退出下载模式。

## 运行

1. 配置 Barrier 服务器以接受您在环境变量 `SCREEN_NAME` 中设置的屏幕名称，并确保关闭 TLS。
2. 将开发板插入 USB 端口。
3. LED 应在启动时闪烁红色，然后在开发板连接到 WiFi 后变为闪烁蓝色，最终在开发板连接到 Barrier 服务器后变为闪烁暗黄色。
4. 当 Barrier 进入屏幕时，LED 变为绿色，当 Barrier 离开屏幕时，LED 变为闪烁暗黄色。
5. 开发板模拟标准键盘和绝对定位鼠标，应该在任何操作系统中都能正常工作。
6. 使用 USB HID 启动协议，因此即使操作系统没有驱动程序，您也应该能够将开发板用作 BIOS/EFI 中的 USB 键盘/鼠标。

## 更新配置

首先，您需要安装 `esptool.py`，可以使用 `pip install esptool` 安装。有关更多信息，请参阅 [官方文档](https://docs.espressif.com/projects/esptool/en/latest/esp32/installation.html)。

### 准备和更新配置

1. 创建一个 JSON 文件，格式参见 [config.json.example](config.json.example)。
2. 将开发板置于下载模式，然后使用 `esptool.py` 烧录 NVS 分区。
    ```bash
    # 擦除 NVS 分区
    esptool.py --chip esp32s3 --port /dev/ttyACM0 write_flash 0x9000 zero.bin
    # 写入配置
    esptool.py --chip esp32s3 --port /dev/ttyACM0 write_flash 0x9000 /path/to/config.json
    ```
3. 退出下载模式并重置开发板，新配置即会生效。

## 为其他 ESP32S3 开发板构建

* 建议在第一次烧录二进制文件到开发板之前擦除闪存，可以使用 `esptool.py` 或 `cargo-espflash`：
    ```bash
    # 使用 cargo-espflash
    cargo espflash erase-flash --chip esp32s3 --port /dev/ttyACM0
    
    # 使用 esptool.py
    esptool.py --chip esp32s3 --port /dev/ttyACM0 erase_flash
    ```

* 如果开发板上有 RGB LED（WS2812B），可以使用 `smartled` 功能启用 LED，并且需要将环境变量 `SMART_LED_PIN` 设置为正确的引脚编号，在 M5AtomS3/Lite 上是 35，在 M5StampS3 上是 21。
    * 例如，为 ESP32-S3-DevKitC-1 构建和烧录二进制文件 (该开发板的RGB LED在IO38)：
        ```bash
        SMART_LED_PIN=38 cargo run --release --features smartled
        ```

* 如果开发板上有普通 LED，可以使用 `led` 功能启用它，并且需要将环境变量 `LED_PIN` 设置为正确的引脚编号。

* 不要同时启用上述多个功能，程序将无法编译。

* 如果未启用上述任何功能，状态指示功能将被禁用。

* 程序将默认输出日志到 UART0，可以使用 `espmonitor` 监控日志。如果开发板没有独立的 UART0 端口，可以禁用默认功能，这将禁用 USB HID 功能，并且您将能够从 USB OTG/J-TAG 端口查看日志。这对于调试与 USB HID 无关的代码非常有用。

## 使用预构建的二进制文件

**注意**：不推荐使用预构建的二进制文件，因为它不能充分利用开发板的全部功能，并且您可能需要更改代码以适应您的开发板。因此，在可能的情况下，您应该自己构建二进制文件。

1. 按照上一节中的描述安装 `esptool.py`。无需安装 Rust 工具链和其他 ESP32 工具。

2. 从 [发布页面](https://github.com/windoze/esparrier/releases) 下载二进制文件。

3. 从压缩包中提取二进制文件。压缩包中有 3 个预构建的二进制文件，选择适合您开发板的那一个。
    * `esparrier.bin` - 适用于大多数具有原生 USB-OTG 端口的通用 ESP32S3 开发板，但不支持状态指示功能。
    * `esparrier-m5atoms3-lite.bin` - 适用于 [M5Atom S3 Lite](https://docs.m5stack.com/en/core/AtomS3%20Lite)。
    * `esparrier-xiao-esp32s3.bin` - 适用于 [Seeed Studio XIAO ESP32S3](https://wiki.seeedstudio.com/xiao_esp32s3_getting_started/)。

4. 按照上一节中的描述准备 `config.json` 文件。

5. 将开发板置于下载模式，然后烧录二进制文件和配置到开发板上。注意 USB 设备名称可能会有所不同，您可能需要将其更改为正确的名称。在大多数 Linux 系统中，设备名称为 `/dev/ttyACMx`，其中 `x` 是一个数字，您可以通过运行 `ls /dev/ttyACM*` 找到正确的设备名称。
    ```bash
    # 擦除全部闪存
    esptool.py --chip esp32s3 --port /dev/ttyACM0 erase_flash
    # 写入二进制文件和配置
    esptool.py --chip esp32s3 --port /dev/ttyACM0 write_flash 0x10000 /path/to/esparrier.bin 0x9000 /path/to/config.json
    ```

6. 退出下载模式并重置开发板，您应该会在主机上看到新的 USB HID 设备。

## 注意事项：

**警告**：此程序仅用于测试目的。它不是 Barrier 客户端的完整实现。可能存在许多错误和缺失的功能。它没有任何安全保障，无论是在 WiFi 还是 USB 上。所以建议仅在私有的安全环境中使用。

* 此代码在 [M5Atom S3 Lite](https://docs.m5stack.com/en/core/AtomS3%20Lite) 上开发和测试，其他 ESP32S3 开发板可能无法工作，或者您需要更改代码以适应您的开发板。
* 强烈建议使用带有外部天线的开发板，ESP32S3 仅支持 2.4G WiFi，而这个频段非常拥挤，如果无线连接不稳定，您可能会遇到抖动和延迟。
* 代码不适用于 ESP8266/ESP32/ESP32C3，因为它们没有所需的 USB 功能，ESP32S2 可能可以通过一些代码适配工作，但未经过测试。
* 不支持 TLS，因此您必须在 Barrier 服务器端禁用 TLS 。
* 鼠标配置为绝对定位模式，您必须在构建前设置正确的屏幕分辨率，否则鼠标可能无法正常工作。
* 由于技术限制，不支持剪贴板、文件传输和跨屏幕拖放，标准 USB HID 设备无法做到这一点。
* 如果未能正确设置屏幕尺寸，自动切换可能无法正常工作，此时需要在 Barrier 服务器上配置热键以手动切换屏幕。
* 频繁的连接/断开可能导致开发板无法连接到 WiFi 和/或 Barrier 服务器，此时需要切断电源并等待几秒钟之后再尝试。
* 理论上，开发板应该也能与 [InputLeap](https://github.com/input-leap/input-leap) 服务器一起工作，但未经测试。
* USB VID/PID 是随机选择的，并未在标准组织和机构注册，您也并未从作者处得到生产和销售使用这些 VID/PID 的 USB 设备的授权，因此您可能需要更改代码以使用您自己的 VID/PID。
* USB 远程唤醒可能无法工作，因为USB标准禁止挂起设备消耗过多电流，但此程序需要比标准规定的更多电流来保持 Wi-Fi 连接。我尚未找到在电流 <2.5mA 的情况下保持程序运行的方法。当然，您可以选择带有外部电源（如电池）的开发板，但这似乎有点小题大做。
* 开发板只有在成功连接到 WiFi 和 Barrier 服务器后才能接受输入，这段延迟可能已经超过了电脑启动时进入 BIOS/EFI 设置的时限，一些主板上带有“始终供电”的USB口也许能避免这个问题，但未经测试，或者您可以使用一个即使主机关闭也能供电的 USB 集线器。
* 如果在定义的 `WATCHDOG_TIMEOUT` 环境变量中的秒数内没有从 Barrier 服务器接收到心跳，或者程序本身失控且未处理心跳，watchdog 将重置开发板。默认的 watchdog 超时时间为 15 秒，因为默认的 Barrier 心跳间隔为 5 秒，如果 Barrier 服务器设置了较长的心跳间隔，您可能需要更改 watchdog 超时时间。
* 剪贴板共享功能可以通过 [ClipSync 应用](https://github.com/windoze/clip-sync) 或操作系统内置的剪贴板共享功能实现，例如 [Windows 上的 Clip Sync](https://support.microsoft.com/en-us/windows/about-the-clipboard-in-windows-c436501e-985d-1c8d-97ea-fe46ddf338c6) 或 [macOS 上的通用剪贴板](https://support.apple.com/guide/mac-help/copy-and-paste-between-devices-mchl70368996/mac)，但此程序中未实现。

## 待办事项：

- [x] 支持媒体键
- [x] 无需重建即可更新配置
- [x] 支持其他 ESP32S3 开发板
- [ ] 支持 Mac 特殊键
- [ ] 支持 TLS
- [ ] NVS 加密
- [ ] OTA 更新
- [ ] 支持剪贴板，也许可以通过在主机上运行的单独应用程序来处理剪贴板数据

## 许可证和版权

* 本项目以 [MIT 许可证](LICENSE) 发布。
* `esp_hal_smartled.rs` 文件取自 [esp-hal-community 仓库](https://github.com/esp-rs/esp-hal-community)，该文件以 MIT 许可证和 Apache 许可证 2.0 版本发布。
* `gentable.c` 中的一些代码片段取自 [Barrier 仓库](https://github.com/debauchee/barrier) 并以 GPLv2 许可证发布。主项目仅使用其输出，而不是代码本身，所以不受GPL许可协议的约束。