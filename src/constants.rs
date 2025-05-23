use const_str::{parse, split};

const VERSION_SEGMENTS: [&str; 3] = split!(env!("CARGO_PKG_VERSION"), ".");
pub const VERSION_MAJOR: u8 = parse!(VERSION_SEGMENTS[0], u8);
pub const VERSION_MINOR: u8 = parse!(VERSION_SEGMENTS[1], u8);
pub const VERSION_PATCH: u8 = parse!(VERSION_SEGMENTS[2], u8);

cfg_if::cfg_if! {
    if #[cfg(feature = "led")] {
        const INDICATOR_FLAGS: u8 = 0b0000_0001;
    }
    else if #[cfg(feature = "smartled")] {
        const INDICATOR_FLAGS: u8 = 0b0000_0010;
    }
    else if #[cfg(feature = "graphics")] {
        const INDICATOR_FLAGS: u8 = 0b0000_0100;
    }
    else {
        const INDICATOR_FLAGS: u8 = 0b0000_0000;
    }
}
cfg_if::cfg_if! {
    if #[cfg(feature = "clipboard")] {
        const CLIPBOARD_FLAG: u8 = 0b1000_0000;
    }
    else {
        const CLIPBOARD_FLAG: u8 = 0b0000_0000;
    }
}
pub const FEATURE_FLAGS: u8 = INDICATOR_FLAGS | CLIPBOARD_FLAG;

cfg_if::cfg_if! {
    if #[cfg(feature = "xiao-esp32s3")] {
        pub const LED_PIN: u8 = 21;
    } else if #[cfg(feature = "led")] {
        #[const_env::from_env]
        pub const LED_PIN: u8 = 0;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "m5atoms3-lite")] {
        pub const SMART_LED_PIN: u8 = 35;
    } else if #[cfg(feature = "devkitc-1_0")] {
        pub const SMART_LED_PIN: u8 = 48;
    } else if #[cfg(feature = "devkitc-1_1")] {
        pub const SMART_LED_PIN: u8 = 38;
    } else if #[cfg(feature = "smartled")] {
        #[const_env::from_env]
        pub const SMART_LED_PIN: u8 = 0;
    }
}

#[cfg(feature = "smartled")]
#[const_env::from_env]
pub const SMART_LED_COUNT: usize = 1;

cfg_if::cfg_if! {
    if #[cfg(feature = "m5atoms3")] {
        pub const PASTE_BUTTON_PIN: u8 = 41;
    } else if #[cfg(feature = "m5atoms3r")] {
        pub const PASTE_BUTTON_PIN: u8 = 41;
    } else if #[cfg(feature = "m5atoms3-lite")] {
        pub const PASTE_BUTTON_PIN: u8 = 41;
    } else if #[cfg(feature = "clipboard")] {
        #[const_env::from_env]
        pub const PASTE_BUTTON_PIN: u8 = 0;
    }
}

#[cfg(feature = "clipboard")]
pub const MAX_CLIPBOARD_SIZE: usize = 1024;

// Default config settings
pub const WIFI_SSID: &str = "my-ssid";
pub const WIFI_PASSWORD: &str = "my-password";
pub const BARRIER_SERVER: &str = "192.168.100.200:24800";
pub const SCREEN_NAME: &str = "my-screen";
pub const SCREEN_WIDTH: u16 = 1920;
pub const SCREEN_HEIGHT: u16 = 1080;
pub const JIGGLE_INTERVAL: u16 = 60;
pub const POLLING_RATE: u16 = 250;
pub const REVERSED_WHEEL: bool = false;

cfg_if::cfg_if! {
    if #[cfg(feature = "graphics")] {
        // 30 is too dim for LCD display
        pub const BRIGHTNESS: u8 = 50;
    } else {
        // But is good for SmartLED
        pub const BRIGHTNESS: u8 = 30;
    }
}

pub const USB_VID: u16 = 0x0d0a;
pub const USB_PID: u16 = 0xc0de;
pub const USB_MANUFACTURER: &str = "0d0a.com";
pub const USB_PRODUCT: &str = "Esparrier KVM";
pub const USB_SERIAL_NUMBER: &str = "88888888";

// WinUSB GUID
pub const DEVICE_INTERFACE_GUIDS: &[&str] = &["{4d36e96c-e325-11ce-bfc1-08002be10318}"];

#[allow(dead_code)]
mod firmware_kinds {
    pub(super) const FIRMWARE_KIND_M5ATOMS3: u8 = 1;
    pub(super) const FIRMWARE_KIND_M5ATOMS3R: u8 = 2;
    pub(super) const FIRMWARE_KIND_M5ATOMS3LITE: u8 = 3;
    pub(super) const FIRMWARE_KIND_XIAO_ESP32S3: u8 = 4;
    pub(super) const FIRMWARE_KIND_DEVKITC_1_0: u8 = 5;
    pub(super) const FIRMWARE_KIND_DEVKITC_1_1: u8 = 6;
    pub(super) const FIRMWARE_KIND_GENERIC: u8 = 0;
    pub(super) const FIRMWARE_KIND_CUSTOM: u8 = 0xFF;
}

cfg_if::cfg_if! {
    if #[cfg(feature = "m5atoms3")] {
        pub const FIRMWARE_KIND: u8 = firmware_kinds::FIRMWARE_KIND_M5ATOMS3;
    } else if #[cfg(feature = "m5atoms3r")] {
        pub const FIRMWARE_KIND: u8 = firmware_kinds::FIRMWARE_KIND_M5ATOMS3R;
    } else if #[cfg(feature = "m5atoms3-lite")] {
        pub const FIRMWARE_KIND: u8 = firmware_kinds::FIRMWARE_KIND_M5ATOMS3LITE;
    } else if #[cfg(feature = "xiao-esp32s3")] {
        pub const FIRMWARE_KIND: u8 = firmware_kinds::FIRMWARE_KIND_XIAO_ESP32S3;
    } else if #[cfg(feature = "devkitc-1_0")] {
        pub const FIRMWARE_KIND: u8 = firmware_kinds::FIRMWARE_KIND_DEVKITC_1_0;
    } else if #[cfg(feature = "devkitc-1_1")] {
        pub const FIRMWARE_KIND: u8 = firmware_kinds::FIRMWARE_KIND_DEVKITC_1_1;
    } else {
        // Default generic firmware does not enable any features
        // This firmware has been customized if any feature is enabled, thus cannot be OTA updated
        pub const FIRMWARE_KIND: u8 = if FEATURE_FLAGS==0 {
            firmware_kinds::FIRMWARE_KIND_GENERIC
        } else {
            firmware_kinds::FIRMWARE_KIND_CUSTOM
        };
    }
}
